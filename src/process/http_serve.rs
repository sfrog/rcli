use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, prot: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], prot));
    info!("Serving directory {:?} on {}", &path, &addr);

    let state = HttpServeState { path };

    // axum router
    let router = Router::new()
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let full_path = state.path.join(path);
    info!("Serving file {:?}", &full_path);

    if full_path.exists() && full_path.is_file() {
        match tokio::fs::read_to_string(full_path).await {
            Ok(content) => {
                info!("File served successfully, read {} bytes", content.len());
                (StatusCode::OK, content)
            }
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("File {} not found", &full_path.display()),
        )
    }
}
