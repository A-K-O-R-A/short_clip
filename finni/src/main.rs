use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;

use http_body_util::{Full, StreamBody};
use hyper::body::{Body, Bytes};
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

use tokio::fs::File;
use tokio::io::AsyncReadExt;

fn auth_denied() -> Response<Full<Bytes>> {
    Response::builder()
        .status(403)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

fn not_found() -> Response<Full<Bytes>> {
    Response::builder()
        .status(404)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

fn internal_error(e: Box<dyn std::error::Error>) -> Response<Full<Bytes>> {
    eprintln!("{e}");

    Response::builder()
        .status(418)
        .body(Full::new(Bytes::from("I'm a teapot")))
        .unwrap()
}

fn content_path() -> PathBuf {
    PathBuf::from(std::env::current_dir().unwrap()).join("contents")
}

async fn handle_req(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(
        match match req.method() {
            &Method::GET => handle_download(req).await,
            &Method::POST => handle_upload(req).await,
            _ => Ok(not_found()),
        } {
            Ok(r) => r,
            Err(e) => internal_error(e),
        },
    )
}

async fn handle_download(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
    let path = req.uri().path();
    if path.len() < 2 {
        return Ok(not_found());
    }
    let short = path[1..].to_lowercase();
    let mut dir_entries = tokio::fs::read_dir(content_path()).await?;

    while let Some(e) = dir_entries.next_entry().await? {
        let file_name = e.file_name().to_str().unwrap().to_lowercase();
        let (name, ext) = file_name.split_once(".").unwrap();
        if name == short {
            let mut f = File::open(e.path()).await?;
            let mut v: Vec<u8> = Vec::with_capacity(f.metadata().await?.len() as usize);
            f.read_to_end(&mut v).await?;

            let resp = Response::builder()
                .header("Content-Type", ext)
                .body(Full::new(Bytes::from(v)))?;

            return Ok(resp);
        }
    }

    Ok(not_found())
}

async fn handle_upload(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
    if let Some(auth) = req.headers().get("Authorization") {
        if auth != "uwu" {
            return Ok(auth_denied());
        }

        if let Some(ext) = req.headers().get("File-Extension") {
            let ext = ext.to_str()?.to_owned();

            let frame_stream = req.into_body();
            let data = frame_stream.collect().await?;

            let raw_data = &data.to_bytes()[..];
            let mut hasher = FxHasher::default();
            hasher.write(raw_data);

            let hash = hasher.finish();
            let short = general_purpose::URL_SAFE_NO_PAD.encode(hash.to_le_bytes());

            let path = content_path().join(&(short.clone() + "." + &ext));

            // Only write file if it wasnt already saved
            if let Err(_) = File::open(&path).await {
                tokio::fs::write(&path, raw_data).await?;
            }

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
