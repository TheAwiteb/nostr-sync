use std::net::SocketAddr;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;

mod error;
mod handler;
mod html;
mod util;

use crate::config::Config;
use crate::error::Error;
use crate::state::SharedState;

fn build(state: SharedState) -> Router {
    Router::new()
        // Static assets (public)
        //.route("/static/img/logo.svg", get(handler::get_logo))
        .route("/static/css/style.css", get(handler::get_style))
        .route("/static/css/pico.min.css", get(handler::get_pico_style))
        .route("/", get(handler::dashboard::get))
        .route(
            "/user/add",
            get(handler::add_user::get).post(handler::add_user::post),
        )
        .fallback(handler::not_found::get)
        .with_state(state)
}

pub async fn serve_web_ui(config: &Config, state: SharedState) -> Result<(), Error> {
    let listen_add: SocketAddr = config.web.listen_addr;

    // Build router
    let router: Router = build(state);

    tracing::debug!("Starting Web UI server...");

    // Bind listener
    let listener: TcpListener = TcpListener::bind(listen_add).await?;

    tracing::info!("Serving Web UI on http://{listen_add}/");

    // Serve web UI
    axum::serve(listener, router).await?;

    Ok(())
}
