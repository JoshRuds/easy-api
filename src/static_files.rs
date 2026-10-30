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

    // Prevent requests from escaping the configured directory.
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

    if relative.is_empty() || relative.ends_with('/') {
        file_path.push("index.html");
    }

    let file = match tokio::fs::read(&file_path).await {
        Ok(file) => file,
        Err(_) => {
            return Some(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("Not Found")))
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
            .status(StatusCode::OK)
            .header("Content-Type", content_type)
            .body(Full::new(body))
            .unwrap(),
    )
}

pub fn serve_static(prefix: &str, directory: impl Into<PathBuf>) {
    let prefix = if prefix == "/" {"/".to_string()} else {format!("/{}", prefix.trim_matches('/'))};
    *STATIC_FILES.write().unwrap() = Some(StaticConfig { prefix, directory: directory.into() });
}