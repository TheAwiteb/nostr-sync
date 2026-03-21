use std::collections::hash_map::Entry;

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use nostr::key::PublicKey;
use serde::Deserialize;

use crate::router::error::RouterError;
use crate::router::html::HtmlTemplate;
use crate::router::util;
use crate::state::{PublicKeySyncData, SharedState};

#[derive(Default, Template)]
#[template(path = "add_user.html")]
struct AddUserTemplate<'a> {
    error: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct AddUserForm {
    public_key: String,
}

pub async fn get() -> Response {
    let template = AddUserTemplate::default();
    HtmlTemplate::new(template).into_response()
}

pub async fn post(
    state: State<SharedState>,
    Form(form): Form<AddUserForm>,
) -> Result<Response, RouterError> {
    // Parse public key
    let Ok(public_key) = PublicKey::parse(&form.public_key) else {
        let template = AddUserTemplate {
            error: Some("Invalid public key."),
        };
        return Ok(HtmlTemplate::new(template).into_response());
    };

    let mut pks = state.pubkeys.write().await;

    match pks.entry(public_key) {
        Entry::Occupied(_) => {
            let template = AddUserTemplate {
                error: Some("Public key already exists."),
            };
            return Ok(HtmlTemplate::new(template).into_response());
        }
        Entry::Vacant(entry) => {
            entry.insert(PublicKeySyncData::default());
        }
    }

    // Go back to dashboard
    Ok(util::redirect_to_dashboard().into_response())
}
