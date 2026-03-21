use axum::response::Redirect;

#[inline]
pub fn redirect_to_dashboard() -> Redirect {
    Redirect::to("/")
}
