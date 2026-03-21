use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::router::html::HtmlTemplate;

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate;

pub async fn get() -> Response {
    let template = NotFoundTemplate;
    (StatusCode::NOT_FOUND, HtmlTemplate::new(template)).into_response()
}
