use std::sync::Arc;

use axum::response::{IntoResponse, Html};
use hyper::StatusCode;

use axum::Extension;
use minijinja_autoreload::AutoReloader;

use crate::mvc::utils::render_template;

pub async fn page_not_found(
    Extension(template_loader): Extension<Arc<AutoReloader>>,
) -> impl IntoResponse {
    let (status, text) = render_template(template_loader, "page_not_found.j2");
    
    let status_result = if status == StatusCode::INTERNAL_SERVER_ERROR {
        status
    } else {
        StatusCode::NOT_FOUND
    };

    (status_result, Html(text))
}

pub async fn internal_error(Extension(template_loader): Extension<Arc<AutoReloader>>) -> impl IntoResponse {
    let (_, text) = render_template(template_loader, "internal_error.j2");

    (StatusCode::INTERNAL_SERVER_ERROR, Html(text));
}