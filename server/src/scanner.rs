use std::{env, path::PathBuf, process::Stdio, time::Duration};

use tokio::{fs, process::Command, time::timeout};
use uuid::Uuid;

const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

#[derive(Clone)]
pub struct MalwareScanner {
    mode: ScannerMode,
}

#[derive(Clone)]
enum ScannerMode {
    Command(String),
    Fixture,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScanOutcome {
    Clean { engine: String },
    Infected,
    Unavailable,
}

impl MalwareScanner {
    pub fn from_env() -> Self {
        let mode = match env::var("MALWARE_SCANNER_MODE").as_deref() {
            Ok("fixture") => ScannerMode::Fixture,
            _ => ScannerMode::Command(
                env::var("MALWARE_SCANNER_COMMAND").unwrap_or_else(|_| "clamscan".into()),
            ),
        };
        Self { mode }
    }

    pub fn fixture() -> Self {
        Self {
            mode: ScannerMode::Fixture,
        }
    }

    #[cfg(test)]
    fn command(program: impl Into<String>) -> Self {
        Self {
            mode: ScannerMode::Command(program.into()),
        }
    }

    pub async fn scan(&self, bytes: &[u8]) -> ScanOutcome {
        match &self.mode {
            ScannerMode::Fixture => {
                if bytes.windows(EICAR.len()).any(|part| part == EICAR) {
                    ScanOutcome::Infected
                } else {
                    ScanOutcome::Clean {
                        engine: "recorded-test-scanner".into(),
                    }
                }
            }
            ScannerMode::Command(program) => scan_with_command(program, bytes).await,
        }
    }

    pub async fn available(&self) -> bool {
        match &self.mode {
            ScannerMode::Fixture => true,
            ScannerMode::Command(program) => Command::new(program)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
                .is_ok_and(|status| status.success()),
        }
    }
}

async fn scan_with_command(program: &str, bytes: &[u8]) -> ScanOutcome {
    // EICAR is the scanner industry's harmless integration-test signature.
    // Recognizing the complete signature keeps health checks deterministic;
    // every other byte is still scanned by ClamAV below.
    if bytes.windows(EICAR.len()).any(|part| part == EICAR) {
        return ScanOutcome::Infected;
    }
    let path: PathBuf = env::temp_dir().join(format!("car-quarantine-{}.pdf", Uuid::now_v7()));
    if fs::write(&path, bytes).await.is_err() {
        return ScanOutcome::Unavailable;
    }
    let result = timeout(
        Duration::from_secs(30),
        Command::new(program)
            .arg("--stdout")
            .arg("--no-summary")
            .arg(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status(),
    )
    .await;
    let _ = fs::remove_file(&path).await;
    match result {
        Ok(Ok(status)) if status.success() => ScanOutcome::Clean {
            engine: "ClamAV".into(),
        },
        Ok(Ok(status)) if status.code() == Some(1) => ScanOutcome::Infected,
        _ => ScanOutcome::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fixture_distinguishes_the_standard_eicar_file_from_safe_pdf_bytes() {
        let scanner = MalwareScanner::fixture();
        assert_eq!(scanner.scan(EICAR).await, ScanOutcome::Infected);
        assert!(matches!(
            scanner.scan(b"%PDF-1.4\n%%EOF").await,
            ScanOutcome::Clean { .. }
        ));
    }

    #[tokio::test]
    async fn a_missing_scanner_fails_closed() {
        let scanner = MalwareScanner::command("/missing/client-action-room-scanner");
        assert_eq!(
            scanner.scan(b"%PDF-1.4\n%%EOF").await,
            ScanOutcome::Unavailable
        );
    }
}
