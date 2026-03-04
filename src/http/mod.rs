pub mod routes;

pub use routes::AppState;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

/// Start the HTTP server on the given port with the provided state
pub async fn start_http_server(port: u16, state: AppState) -> anyhow::Result<()> {
    let app = routes::create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
