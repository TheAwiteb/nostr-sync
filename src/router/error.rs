use std::fmt::Display;

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub title: String,
    pub message: String,
}

/// Router error that renders to an HTML error page
pub struct RouterError {
    status: StatusCode,
    title: String,
    message: String,
}

impl RouterError {
    pub fn internal<E>(error: E) -> Self
    where
        E: Display,
    {
        tracing::error!("Internal error: {error}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            title: String::from("Something went wrong"),
            message: String::from("An internal error occurred."),
        }
    }
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        let template = ErrorTemplate {
            title: self.title,
            message: self.message,
        };

        (self.status, Html(template.to_string())).into_response()
    }
}

impl From<crate::error::Error> for RouterError {
    fn from(error: crate::error::Error) -> Self {
        Self::internal(error)
    }
}
