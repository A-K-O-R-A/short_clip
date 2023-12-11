use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;

use hyper::Method;
use tokio::net::TcpListener;

mod download;
mod upload;
mod util;

use download::handle_download;
use upload::handle_upload;

use crate::util::*;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("PORT").unwrap_or("3000".to_owned());
    let port: u16 = port.parse()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Init authorized token list
    upload::load_authorized_tokens().await?;

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
