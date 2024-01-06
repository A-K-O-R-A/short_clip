use hyper::body::Bytes;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, LOCATION};
use hyper::{Request, Response};

use http_body_util::{BodyExt, Full};
use shared::Metadata;

use base64::{engine::general_purpose, Engine as _};
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::OnceLock;
use tokio::fs;

use crate::util::*;

static TOKENS_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
static HOST: OnceLock<String> = OnceLock::new();

pub async fn initialise_cells() -> Result<(), std::io::Error> {
    let str = fs::read_to_string("./.authorized_tokens").await?;
    let mut map = HashMap::new();

    for line in str.lines() {
        let (token, username) =line.split_once(" ").expect("Invalid file format for .authorized_tokens file. Values should be lines of \"<token> <username>\"");
        map.insert(token.to_owned(), username.to_owned());
    }

    TOKENS_MAP.set(map).unwrap();

    if let Ok(host) = std::env::var("HOST") {
        HOST.set(host).unwrap();
    }

    Ok(())
}

pub async fn handle_upload(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error>> {
    let auth_header = match req.headers().get(AUTHORIZATION) {
        Some(v) => v.to_str()?.to_owned(),
        None => return Ok(bad_request("Missing Authorization header")),
    };
    let content_type = match req.headers().get(CONTENT_TYPE) {
        Some(v) => v.to_str()?.to_owned(),
        None => return Ok(bad_request("Missing Content-Type header")),
    };

    // TODO: Implement actual authentication
    let token_map = TOKENS_MAP.get().unwrap();
    let token = auth_header.trim_start_matches("Bearer ");
    let username = match token_map.get(token) {
        Some(u) => u,
        None => return Ok(auth_denied()),
    };

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
        let metadata = Metadata::new(username, &content_type);
        fs::write(&data_path, raw_data).await?;
        fs::write(&metadata_path, metadata.to_string()?).await?;
    }

    // Return the newly cerated link depending on build
    let redirect = if let Some(host) = HOST.get() {
        format!("https://{host}/{id}")
    } else {
        format!("http://localhost:3000/{id}")
    };

    let resp = Response::builder()
        .status(201) // "Created" Status
        .header(LOCATION, &redirect)
        .body(Full::new(Bytes::from(redirect)))?;

    return Ok(resp);
}
