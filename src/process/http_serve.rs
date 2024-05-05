use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    routing::get,
    Router,
};
use handlebars::{to_json, Handlebars};
use serde::Serialize;
use serde_json::Map;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

#[derive(Serialize)]
struct ServeFile {
    name: String,
    path: String,
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
        .route("/*path", get(path_handler))
        .route("/", get(root_handler))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn root_handler(State(state): State<Arc<HttpServeState>>) -> (StatusCode, HeaderMap, String) {
    file_handler(state, "").await
}

async fn path_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, HeaderMap, String) {
    file_handler(state, &path).await
}

async fn file_handler(
    state: Arc<HttpServeState>,
    pathname: &str,
) -> (StatusCode, HeaderMap, String) {
    let full_path = state.path.join(pathname);
    info!("Serving file {:?}", &full_path);
    let mut header_map = HeaderMap::new();

    if full_path.exists() {
        match read_from_path(full_path, pathname).await {
            Ok((content, content_type)) => {
                header_map.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
                (StatusCode::OK, header_map, content)
            }
            Err(e) => {
                warn!("Error reading path: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    header_map,
                    format!("Error: {}", e),
                )
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            header_map,
            format!("File {} not found", &full_path.display()),
        )
    }
}

async fn read_from_path(path: PathBuf, pathname: &str) -> Result<(String, &'static str)> {
    if path.is_file() {
        let content = tokio::fs::read_to_string(path).await?;
        info!("File served successfully, read {} bytes", content.len());
        Ok((content, "text/plain"))
    } else {
        let content = read_dir_to_html(path, pathname)?;
        info!("Directory served successfully");
        Ok((content, "text/html"))
    }
}

fn read_dir_to_html(path: PathBuf, pathname: &str) -> Result<String> {
    let dir = std::fs::read_dir(path)?;

    let mut files = Vec::new();

    for entry in dir {
        let name = entry?.file_name();
        let pathname = if pathname.is_empty() {
            "".to_owned()
        } else {
            format!("/{}", pathname)
        };

        files.push(ServeFile {
            name: name.to_string_lossy().to_string(),
            path: format!("{}/{}", pathname, name.to_string_lossy()),
        });
    }

    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("index", "./assets/template.hbs")?;

    let mut data = Map::new();
    data.insert("files".to_string(), to_json(files));

    Ok(handlebars.render("index", &data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });

        let path = "Cargo.toml";
        let (status, _, body) = file_handler(state, path).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body.trim().starts_with("[package]"));
    }

    #[tokio::test]
    async fn test_dir_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });

        let path = "fixtures";
        let (status, _, body) = file_handler(state, path).await;

        assert_eq!(status, StatusCode::OK);
        assert!(body
            .trim()
            .contains("<a href='/fixtures/jwt.key'>jwt.key</a>"));
    }
}
