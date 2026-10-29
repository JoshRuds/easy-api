use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, LazyLock, RwLock};
use std::sync::mpsc;

use http_body_util::{Full, BodyExt};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode, Method};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;



type CallbackFuncNoInput = Arc<dyn Fn() + Send + Sync>;
type CallbackFuncTakesInput = Arc<dyn Fn(String) + Send + Sync>;
type CallbackFuncNoInputGivesOutput = Arc<dyn Fn() -> String + Send + Sync>;
type CallbackFuncTakesInputGivesOutput = Arc<dyn Fn(String) -> String + Send + Sync>;

#[derive(Clone)]
enum Callback {
    NoInput(CallbackFuncNoInput),
    TakesInput(CallbackFuncTakesInput),
    NoInputGivesOutput(CallbackFuncNoInputGivesOutput),
    TakesInputGivesOutput(CallbackFuncTakesInputGivesOutput),
}

static API_CALLBACKS: LazyLock<RwLock<HashMap<String, Callback>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

async fn handler(req: Request<Incoming>, ) -> Result<Response<Full<Bytes>>, Infallible> {
    let callback = {
        API_CALLBACKS
            .read()
            .unwrap()
            .get(req.uri().path())
            .cloned()
    };

    match callback {
        Some(Callback::TakesInput(func)) => {
            if req.method() != Method::POST {
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from("Method Not Allowed")))
                    .unwrap());
            }

            let body = match req.into_body().collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Full::new(Bytes::from("Invalid request body")))
                        .unwrap());
                }
            };

            let input = String::from_utf8_lossy(&body).into_owned();
            func(input);

            return Ok(Response::new(Full::new(Bytes::from("OK"))));
        }

        Some(Callback::NoInput(func)) => {
            if req.method() != Method::GET {
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from("Method Not Allowed")))
                    .unwrap());
            }

            func();

            return Ok(Response::new(Full::new(Bytes::from("OK"))));
        }

        Some(Callback::NoInputGivesOutput(func)) => {
            if req.method() != Method::GET {
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from("Method Not Allowed")))
                    .unwrap());
            }

            let output = func();

            return Ok(Response::new(Full::new(Bytes::from(output))));
        }

        Some(Callback::TakesInputGivesOutput(func)) => {
            if req.method() != Method::POST {
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new(Bytes::from("Method Not Allowed")))
                    .unwrap());
            }

            let body = match req.into_body().collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => {
                    return Ok(
                        Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Full::new(Bytes::from(
                                "Invalid request body",
                            )))
                            .unwrap(),
                    );
                }
            };

            let input = String::from_utf8_lossy(&body).into_owned();
            let output = func(input);

            return Ok(Response::new(Full::new(Bytes::from(output))));
        }

        None => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Not found")))
                .unwrap());
        }
    }
}

async fn init(listener: TcpListener) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        .insert(uri.to_string(), Callback::NoInput(Arc::new(func)));
}

pub fn add_grab_api(uri: &str, func: impl Fn(String) + Send + Sync + 'static, ) {
    API_CALLBACKS
        .write()
        .unwrap()
        .insert(uri.to_string(), Callback::TakesInput(Arc::new(func)));
}

pub fn add_tap_give_api(uri: &str, func: impl Fn() -> String + Send + Sync + 'static, ) {
    API_CALLBACKS
        .write()
        .unwrap()
        .insert(uri.to_string(), Callback::NoInputGivesOutput(Arc::new(func)));
}

pub fn add_grab_give_api(uri: &str, func: impl Fn(String) -> String + Send + Sync + 'static, ) {
    API_CALLBACKS
        .write()
        .unwrap()
        .insert(uri.to_string(), Callback::TakesInputGivesOutput(Arc::new(func)));
}

pub fn start() {
    start_on("127.0.0.1:3000");
}

pub fn start_on(address: &str) -> String {
    let addr = address.to_string();
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async move {
            let listener = match TcpListener::bind(&addr).await {
                Ok(listener) => listener,
                Err(err) => {
                    let _ = tx.send(Err(err.to_string()));
                    return;
                }
            };

            let actual_addr = match listener.local_addr() {
                Ok(addr) => addr.to_string(),
                Err(err) => {
                    let _ = tx.send(Err(err.to_string()));
                    return;
                }
            };

            // Tell the caller which address was actually bound.
            if tx.send(Ok(actual_addr)).is_err() {
                return;
            }

            if let Err(err) = init(listener).await {
                eprintln!("Server error: {err}");
            }
        });
    });

    rx.recv()
        .expect("Server thread exited before reporting its address")
        .unwrap_or_else(|err| panic!("Failed to start server: {err}"))
}

pub fn start_on_any() -> String {
    start_on("127.0.0.1:0")
}

pub fn park() {
    std::thread::park();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_registration() {
        add_tap_api("/test", || {
            println!("Callback invoked");
        });

        let callbacks = API_CALLBACKS.read().unwrap();
        assert!(callbacks.contains_key("/test"));
    }
}