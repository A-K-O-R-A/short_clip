use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, LOCATION, X_CONTENT_TYPE_OPTIONS};
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

        // Check if this is a URL
        if metadata.content_type == "text/uri-list" {
            let url = String::from_utf8(v)?;

            let resp = Response::builder()
                // Temporary Redirect
                .status(307)
                .header(LOCATION, &url)
                .header(CONTENT_TYPE, metadata.content_type)
                // Ignore browsers guessing the content type
                .header(X_CONTENT_TYPE_OPTIONS, "nosniff")
                .body(Full::new(Bytes::from(url.into_bytes())))?;

            return Ok(resp);
        }

        // Guess viable file name
        let mime: mime_guess::Mime = metadata.content_type.parse()?;
        let extension = *mime_guess::get_mime_extensions(&mime)
            .unwrap_or(&["bin"])
            .first()
            .unwrap_or(&"bin");
        let _filename = format!("{id}.{extension}");

        // Build response
        let resp = Response::builder()
            .header(CONTENT_TYPE, metadata.content_type)
            // Enable caching
            .header(CACHE_CONTROL, "max-age=31536000, immutable")
            // Ignore browsers guessing the content type
            .header(X_CONTENT_TYPE_OPTIONS, "nosniff")
            // Auto download
            //.header(CONTENT_DISPOSITION,format!("attachment; filename=\"{filename}\""),)
            .body(Full::new(Bytes::from(v)))?;

        return Ok(resp);
    }

    Ok(not_found())
}
