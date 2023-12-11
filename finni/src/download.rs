use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};

use shared::Metadata;

use crate::util::*;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn handle_download(
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
