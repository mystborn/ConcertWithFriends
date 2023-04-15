use std::sync::Arc;

use axum::{Extension, response::{Html, IntoResponse, Response}, extract::Path, body::{self, Full, Empty}};
use hyper::StatusCode;
use minijinja_autoreload::AutoReloader;

use crate::{mvc::utils::render_template, settings::Settings};

pub async fn index(Extension(template_loader): Extension<Arc<AutoReloader>>) -> impl IntoResponse {
    let (status, text) = render_template(template_loader, "index.j2");

    (status, Html(text))
}

pub async fn static_file(Path(path): Path<String>) -> impl IntoResponse {
    let path = format!("static/{}", path.trim_start_matches('/'));
    let mime_type = mime_guess::from_path(&path).first_or_text_plain();
    let file = std::fs::read_to_string(path);
    match file {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(contents)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
    }
}

pub async fn read_env(Extension(settings): Extension<Arc<Settings>>) -> String {
    settings.env.to_string()
}