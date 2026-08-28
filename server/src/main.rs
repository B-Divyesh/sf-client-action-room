use std::{env, net::SocketAddr, time::Duration};

use client_action_room_api::{app, state::AppState};
use tokio::{net::TcpListener, signal, time};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let build_sha = option_env!("BUILD_SHA").unwrap_or("dev");
    let state = AppState::from_env(build_sha)
        .await
        .expect("runtime database configuration must initialize");
    let purge_state = state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3_600));
        loop {
            interval.tick().await;
            match purge_state.purge_expired().await {
                Ok(count) if count > 0 => {
                    info!(expired_demo_sessions = count, "expired demos purged")
                }
                Ok(_) => {}
                Err(error) => error!(%error, "expired demo purge failed"),
            }
        }
    });

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address)
        .await
        .expect("the configured API port must be available");
    info!(
        port,
        build_sha, "client-action-room started; no required environment variables or secrets"
    );

    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("API server stopped unexpectedly");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C handler must install");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler must install")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
