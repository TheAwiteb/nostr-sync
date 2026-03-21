use axum::Router;
use axum::routing::get;

mod error;
mod handler;
mod html;

use crate::state::SharedState;

pub fn build(state: SharedState) -> Router {
    Router::new()
        // Static assets (public)
        //.route("/static/img/logo.svg", get(handler::get_logo))
        .route("/static/css/style.css", get(handler::get_style))
        .route("/static/css/pico.min.css", get(handler::get_pico_style))
        .route("/", get(handler::dashboard::get))
        .fallback(handler::not_found::get)
        .with_state(state)
}
