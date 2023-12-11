use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, LOCATION};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

use http_body_util::BodyExt;
use hyper::Method;
use shared::Metadata;
use tokio::net::TcpListener;

use base64::{engine::general_purpose, Engine as _};
use rustc_hash::FxHasher;
use std::hash::Hasher;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

fn bad_request(msg: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(400)
        .body(Full::new(Bytes::from(msg.to_owned())))
        .unwrap()
}

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
    // Extract id
    let path = req.uri().path();
    if path.len() < 2 {
        return Ok(not_found());
    }
    let id = &path[1..];

    let mut dir_entries = tokio::fs::read_dir(content_path()).await?;

    while let Some(entry) = dir_entries.next_entry().await? {
        if entry.file_name() != id {
            continue;
        }

        // Retrieve metadata to set Content-Type
        let metadata = tokio::fs::read(entry.path().with_extension("json")).await?;
        let metadata = Metadata::from_slice(&metadata)?;

        // Read data
        let mut file = File::open(entry.path()).await?;
        let mut v: Vec<u8> = Vec::with_capacity(file.metadata().await?.len() as usize);
        file.read_to_end(&mut v).await?;

        // Build response
        let resp = Response::builder()
            .header(CONTENT_TYPE, metadata.content_type)
            .body(Full::new(Bytes::from(v)))?;

        return Ok(resp);
    }

    Ok(not_found())
}

async fn handle_upload(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
    let auth = match req.headers().get(AUTHORIZATION) {
        Some(v) => v.to_str()?.to_owned(),
        None => return Ok(bad_request("Missing Authorization header")),
    };
    let content_type = match req.headers().get(CONTENT_TYPE) {
        Some(v) => v.to_str()?.to_owned(),
        None => return Ok(bad_request("Missing Content-Type header")),
    };

    // TODO: Implement actual authentication
    if auth != "uwu" {
        return Ok(auth_denied());
    }

    // Collect all of the data into single Bytes instance
    let frame_stream = req.into_body();
    let data = frame_stream.collect().await?;
    let raw_data = &data.to_bytes()[..];

    // Hash the data
    let mut hasher = FxHasher::default();
    hasher.write(raw_data);
    let hash = hasher.finish();

    // Create short alias for this data
    let id = general_purpose::URL_SAFE_NO_PAD.encode(hash.to_le_bytes());

    let data_path = content_path().join(&id);
    let metadata_path = data_path.with_extension("json");

    // Only write file if it wasnt already saved
    if !data_path.try_exists()? {
        // Save metadata to associate content type
        let metadata = Metadata::new(&auth, &content_type);
        tokio::fs::write(&data_path, raw_data).await?;
        tokio::fs::write(&metadata_path, metadata.to_string()?).await?;
    }

    // Return the newly cerated link
    let redirect = format!("http://localhost:3000/{id}");
    let resp = Response::builder()
        .status(201) // "Created" Status
        .header(LOCATION, &redirect)
        .body(Full::new(Bytes::from(redirect)))?;

    return Ok(resp);
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
