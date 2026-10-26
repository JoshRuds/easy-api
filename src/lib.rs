use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, LazyLock, RwLock};
use std::option::Option;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

type Callback = Arc<dyn Fn() + Send + Sync>;
static API_CALLBACKS: LazyLock<RwLock<HashMap<String, Callback>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

async fn handler(req: Request<Incoming>, ) -> Result<Response<Full<Bytes>>, Infallible> {
    let callback = {
        API_CALLBACKS
            .read()
            .unwrap()
            .get(req.uri().path())
            .cloned()
    };

    if let Some(callback) = callback {
        callback();

        Ok(Response::new(Full::new(Bytes::from("OK"))))

    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not found")))
            .unwrap())
    }
}

async fn init(addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("HTTP error: {err}");
            }
        });
    }
}

pub fn add_tap_api(uri: &str, func: impl Fn() + Send + Sync + 'static, ) {
    API_CALLBACKS
        .write()
        .unwrap()
        .insert(uri.to_string(), Arc::new(func));
}

pub fn start(address: Option<&str>) {
    let addr: String = address.unwrap_or("127.0.0.1:3000").to_string();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            if let Err(err) = init(&addr).await {
                eprintln!("Server error: {err}");
            }
        });
    });
}

pub fn park() {
    std::thread::park();
}