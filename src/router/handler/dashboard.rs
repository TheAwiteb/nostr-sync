use std::collections::BTreeSet;

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use nostr::PublicKey;

use crate::router::error::RouterError;
use crate::router::html::HtmlTemplate;
use crate::state::SharedState;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    // Use the BTreeSet so they are displayed always in the same order
    users: BTreeSet<PublicKey>,
}

pub async fn get(state: State<SharedState>) -> Result<Response, RouterError> {
    let pks = state.pubkeys.read().await;
    let users: BTreeSet<PublicKey> = pks.keys().copied().collect();

    let template = DashboardTemplate { users };

    Ok(HtmlTemplate::new(template).into_response())
}
