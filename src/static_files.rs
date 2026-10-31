use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use tokio;
use http_body_util::{Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode, Method};
use mime_guess;

#[derive(Clone)]
struct StaticConfig {
    prefix: String,
    directory: PathBuf,
}

static STATIC_FILES: LazyLock<RwLock<Option<StaticConfig>>> = LazyLock::new(|| RwLock::new(None));

pub async fn serve_static_file(req: &Request<Incoming>,) -> Option<Response<Full<Bytes>>> {
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return None;
    }

    let config = STATIC_FILES.read().unwrap().clone()?;

    let path = req.uri().path();

    let relative = if config.prefix == "/" {
        path.trim_start_matches('/')
    } else {
        let relative = path.strip_prefix(&config.prefix)?;

        if !relative.is_empty() && !relative.starts_with('/') {
            return None;
        }

        relative.trim_start_matches('/')
    };

    let relative_path = std::path::Path::new(relative);

    // Prevent path traversal.
    if relative_path.components().any(|component| {
        !matches!(component, std::path::Component::Normal(_))
    }) {
        return Some(
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Full::new(Bytes::from("Forbidden")))
                .unwrap(),
        );
    }

    let mut file_path = config.directory.join(relative_path);
    let mut status = StatusCode::OK;

    if relative.is_empty() || relative.ends_with('/') {
        file_path.push("index.html");
    } else {
        // Check if the requested path is a file.
        let is_file = tokio::fs::metadata(&file_path)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false);

        if !is_file {
            // Try directory/index.html.
            let index_path = file_path.join("index.html");

            let has_index = tokio::fs::metadata(&index_path)
                .await
                .map(|m| m.is_file())
                .unwrap_or(false);

            if has_index {
                file_path = index_path;
            } else {
                // Not found: serve custom 404 page.
                file_path = std::path::PathBuf::from(
                    "easy-api/src/errors/404.html"
                );

                status = StatusCode::NOT_FOUND;
            }
        }
    }

    // Read the requested file.
    let file = match tokio::fs::read(&file_path).await {
        Ok(file) => file,
        Err(_) => {
            // If the requested file fails, try the custom 404.
            if status == StatusCode::NOT_FOUND {
                return Some(
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from("404 Not Found")))
                        .unwrap(),
                );
            }

            let error_page = match tokio::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/errors/404.html")).await {
                Ok(file) => file,
                Err(_) => {
                    return Some(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Full::new(Bytes::from("404 Not Found")))
                            .unwrap(),
                    );
                }
            };

            return Some(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Full::new(if req.method() == Method::HEAD {
                        Bytes::new()
                    } else {
                        Bytes::from(error_page)
                    }))
                    .unwrap(),
            );
        }
    };

    let content_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let body = if req.method() == Method::HEAD {
        Bytes::new()
    } else {
        Bytes::from(file)
    };

    Some(
        Response::builder()
            .status(status)
            .header("Content-Type", content_type)
            .body(Full::new(body))
            .unwrap(),
    )
}

pub fn serve_static(prefix: &str, directory: impl Into<PathBuf>) {
    let prefix = if prefix == "/" {"/".to_string()} else {format!("/{}", prefix.trim_matches('/'))};
    *STATIC_FILES.write().unwrap() = Some(StaticConfig { prefix, directory: directory.into() });
}