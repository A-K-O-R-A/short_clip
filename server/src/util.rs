use std::path::PathBuf;

use http_body_util::Full;
use hyper::{body::Bytes, Response};

pub fn bad_request(msg: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(400)
        .body(Full::new(Bytes::from(msg.to_owned())))
        .unwrap()
}

pub fn auth_denied() -> Response<Full<Bytes>> {
    Response::builder()
        .status(403)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

pub fn not_found() -> Response<Full<Bytes>> {
    Response::builder()
        .status(404)
        .body(Full::new(Bytes::new()))
        .unwrap()
}

pub fn internal_error(e: Box<dyn std::error::Error>) -> Response<Full<Bytes>> {
    eprintln!("{e}");

    Response::builder()
        .status(418)
        .body(Full::new(Bytes::from("I'm a teapot")))
        .unwrap()
}

pub fn content_path() -> PathBuf {
    PathBuf::from(std::env::current_dir().expect("Unable to get CWD")).join("contents")
}
