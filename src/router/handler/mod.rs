use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};

pub(super) mod dashboard;
pub(super) mod not_found;

const PICO_STYLE: &str = include_str!("../../../static/css/pico.min.css");
const STYLE: &str = include_str!("../../../static/css/style.css");
// const LOGO: &str = include_str!("../../../static/img/logo.svg");

// pub(super) async fn get_logo() -> Response {
//     ([(CONTENT_TYPE, "image/svg+xml")], LOGO).into_response()
// }

pub(super) async fn get_style() -> Response {
    ([(CONTENT_TYPE, "text/css")], STYLE).into_response()
}

pub(super) async fn get_pico_style() -> Response {
    ([(CONTENT_TYPE, "text/css")], PICO_STYLE).into_response()
}
