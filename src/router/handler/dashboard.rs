use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::router::error::RouterError;
use crate::router::html::HtmlTemplate;
use crate::state::SharedState;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {}

pub async fn get(_state: State<SharedState>) -> Result<Response, RouterError> {
    let template = DashboardTemplate {};

    Ok(HtmlTemplate::new(template).into_response())
}
