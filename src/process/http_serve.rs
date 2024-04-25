use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, prot: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], prot));
    info!("Serving directory {:?} on {}", &path, &addr);

    let state = HttpServeState { path: path.clone() };

    let dir_service = ServeDir::new(path)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_zstd();

    // axum router
    let router = Router::new()
        .nest_service("/tower", dir_service)
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

#[cfg(test)]
mod tests {
    use super::*;
    // use axum::http::{Request, Response};
    // use std::net::SocketAddr;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });

        let path = Path("Cargo.toml".to_string());
        let (status, content) = file_handler(State(state), path).await;

        assert_eq!(status, StatusCode::OK);
        assert!(content.trim().starts_with("[package]"));
    }

    // #[tokio::test]
    // async fn test_file_handler_not_found() {
    //     let state = Arc::new(HttpServeState {
    //         path: PathBuf::from("tests"),
    //     });

    //     let path = Path::from("not_found.txt");
    //     let (status, content) = file_handler(State(state), path).await;

    //     assert_eq!(status, StatusCode::NOT_FOUND);
    //     assert_eq!(content, "File tests/not_found.txt not found");
    // }

    // #[tokio::test]
    // async fn test_file_handler_error() {
    //     let state = Arc::new(HttpServeState {
    //         path: PathBuf::from("tests"),
    //     });

    //     let path = Path::from("error.txt");
    //     let (status, content) = file_handler(State(state), path).await;

    //     assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    //     assert!(content.contains("Error:"));
    // }

    // #[tokio::test]
    // async fn test_process_http_serve() {
    //     let path = PathBuf::from("tests");
    //     let prot = 8080;
    //     let addr = SocketAddr::from(([127, 0, 0, 1], prot));

    //     let server = tokio::spawn(async move {
    //         process_http_serve(path, prot).await.unwrap();
    //     });

    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    //     let client = reqwest::Client::new();
    //     let response = client
    //         .get(format!("http://{}:{}/test.txt", addr.ip(), addr.port()))
    //         .send()
    //         .await
    //         .unwrap();
    //     assert_eq!(response.status(), StatusCode::OK);
    //     assert_eq!(response.text().await.unwrap(), "Hello, world!\n");

    //     server.abort();
    // }
}
