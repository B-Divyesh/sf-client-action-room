use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::auth::AuthService;
use crate::scanner::MalwareScanner;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub build_sha: String,
    pub pool: SqlitePool,
    pub limiter: RateLimiter,
    pub fixed_now: Option<DateTime<Utc>>,
    pub dist_dir: PathBuf,
    pub auth: AuthService,
    pub scanner: MalwareScanner,
    database_path: Option<PathBuf>,
    persist_path: Option<PathBuf>,
}

impl AppState {
    pub async fn from_env(build_sha: impl Into<String>) -> Result<Self> {
        let (database_url, source, database_path, persist_path) = database_url()?;
        let options = SqliteConnectOptions::from_str(&database_url)
            .context("DATABASE_URL must be a valid SQLite URL")?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(5));
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("demo database must open")?;
        sqlx::migrate!()
            .run(&pool)
            .await
            .context("demo database migrations must run")?;

        let fixed_now = env::var("DEMO_FIXED_NOW")
            .ok()
            .map(|value| {
                DateTime::parse_from_rfc3339(&value)
                    .map(|parsed| parsed.with_timezone(&Utc))
                    .context("DEMO_FIXED_NOW must be RFC 3339")
            })
            .transpose()?;
        let dist_dir = env::var_os("DIST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("dist"));

        info!(
            database_config = source,
            fixed_clock = fixed_now.is_some(),
            "runtime configuration loaded; no secret values logged"
        );

        let mut state = Self::new(build_sha, pool, fixed_now, dist_dir);
        state.auth = AuthService::from_env();
        state.scanner = MalwareScanner::from_env();
        state.database_path = database_path;
        state.persist_path = persist_path;
        state.persist_snapshot().await?;
        Ok(state)
    }

    pub fn new(
        build_sha: impl Into<String>,
        pool: SqlitePool,
        fixed_now: Option<DateTime<Utc>>,
        dist_dir: PathBuf,
    ) -> Self {
        Self::new_with_persistence(build_sha, pool, fixed_now, dist_dir, None, None)
    }

    pub fn new_with_persistence(
        build_sha: impl Into<String>,
        pool: SqlitePool,
        fixed_now: Option<DateTime<Utc>>,
        dist_dir: PathBuf,
        database_path: Option<PathBuf>,
        persist_path: Option<PathBuf>,
    ) -> Self {
        Self {
            build_sha: build_sha.into(),
            pool,
            limiter: RateLimiter::default(),
            fixed_now,
            dist_dir,
            auth: AuthService::for_tests(),
            scanner: MalwareScanner::fixture(),
            database_path,
            persist_path,
        }
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.fixed_now.unwrap_or_else(Utc::now)
    }

    pub async fn purge_expired(&self) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM workspaces WHERE namespace = 'demo' AND expires_at <= ?")
                .bind(self.now().to_rfc3339())
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected())
    }

    pub async fn persist_snapshot(&self) -> Result<()> {
        let (Some(database), Some(persist)) = (&self.database_path, &self.persist_path) else {
            return Ok(());
        };
        sqlx::query("PRAGMA wal_checkpoint(FULL)")
            .execute(&self.pool)
            .await
            .ok();
        let temporary = persist.with_extension("sqlite3.next");
        let bytes = fs::read(database).context("database snapshot must read")?;
        fs::write(&temporary, bytes).context("database snapshot must write")?;
        fs::rename(&temporary, persist).context("database snapshot must publish")?;
        Ok(())
    }
}

fn database_url() -> Result<(String, &'static str, Option<PathBuf>, Option<PathBuf>)> {
    if let Ok(value) = env::var("DATABASE_URL") {
        return Ok((value, "supplied", None, None));
    }

    let preferred = env::var_os("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/data"));
    let directory = writable_directory(&preferred).unwrap_or_else(|| PathBuf::from("."));
    let database = directory.join("client-action-room.sqlite3");
    let persist_path = env::var_os("PERSIST_DIR")
        .map(PathBuf::from)
        .map(|directory| {
            let _ = fs::create_dir_all(&directory);
            directory.join("client-action-room.sqlite3")
        });
    if let Some(persist) = &persist_path {
        if persist.exists() && !database.exists() {
            fs::copy(persist, &database).context("persistent database snapshot must restore")?;
        }
    }
    Ok((
        sqlite_url(&database),
        "generated-default",
        Some(database),
        persist_path,
    ))
}

fn writable_directory(path: &Path) -> Option<PathBuf> {
    if fs::create_dir_all(path).is_err() {
        return None;
    }
    let probe = path.join(".car-write-probe");
    match fs::write(&probe, b"") {
        Ok(()) => {
            let _ = fs::remove_file(probe);
            Some(path.to_path_buf())
        }
        Err(_) => None,
    }
}

fn sqlite_url(path: &Path) -> String {
    format!("sqlite://{}", path.to_string_lossy())
}

#[derive(Clone, Default)]
pub struct RateLimiter {
    events: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl RateLimiter {
    pub fn check(&self, key: String, allowance: usize, window: Duration) -> Result<(), u64> {
        let now = Instant::now();
        let mut events = self.events.lock().expect("rate limiter lock poisoned");
        let entries = events.entry(key).or_default();
        while entries
            .front()
            .is_some_and(|seen| now.duration_since(*seen) >= window)
        {
            entries.pop_front();
        }

        if entries.len() >= allowance {
            let elapsed = entries
                .front()
                .map(|seen| now.duration_since(*seen))
                .unwrap_or_default();
            let retry = window.saturating_sub(elapsed).as_secs().max(1);
            return Err(retry);
        }

        entries.push_back(now);
        if events.len() > 10_000 {
            events.retain(|_, values| values.back().is_some_and(|seen| seen.elapsed() < window));
        }
        Ok(())
    }
}
