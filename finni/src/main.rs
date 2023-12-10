use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

use http_body_util::BodyExt;
use hyper::Method;
use tokio::net::TcpListener;

use base64::{engine::general_purpose, Engine as _};
use rustc_hash::FxHasher;
use std::hash::Hasher;

fn auth_denied() -> Response<Full<Bytes>> {
    Response::builder()
        .status(403)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

fn content_path(short: &str) -> PathBuf {
    PathBuf::from(std::env::current_dir().unwrap())
        .join("contents")
        .join(short)
}

async fn handle_req(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.method() {
        &Method::GET => handle_download(req).await,
        &Method::POST => handle_upload(req).await,
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::new()))
            .unwrap()),
    }
}

async fn handle_download(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn handle_upload(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    if let Some(auth) = req.headers().get("Authorization") {
        if auth != "uwu" {
            return Ok(auth_denied());
        }

        if let Some(_ext) = req.headers().get("File-Extension") {
            let frame_stream = req.into_body();
            let data = frame_stream.collect().await.unwrap();
            let short = tokio::task::spawn_blocking(move || {
                let raw_data = &data.to_bytes()[..];
                let mut hasher = FxHasher::default();
                hasher.write(raw_data);

                let hash = hasher.finish();
                let short = general_purpose::URL_SAFE.encode(hash.to_le_bytes());

                // Only write file if it wasnt already saved
                if let Err(_) = std::fs::File::open(&short) {
                    std::fs::write(content_path(&short), raw_data).unwrap();
                }

                short
            })
            .await
            .unwrap();

            return Ok(Response::new(Full::new(Bytes::from(format!(
                "http://localhost:3000/{short}"
            )))));
        }

        Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
    } else {
        Ok(auth_denied())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handle_req))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
