use hyper::body::Bytes;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, LOCATION};
use hyper::{Request, Response};

use http_body_util::{BodyExt, Full};
use shared::Metadata;

use base64::{engine::general_purpose, Engine as _};
use rustc_hash::FxHasher;
use std::hash::Hasher;

use crate::util::*;

pub async fn handle_upload(
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
