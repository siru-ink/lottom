use crate::{
    AppState,
    db::item::Item,
    template::{ItemPage, NotFoundPage},
};
use axum::{
    extract::{Query, State},
    response::Response,
};
use std::{collections::HashMap, sync::Arc};

pub async fn get_item(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let item_id_unparsed = match params.get("item_id") {
        Some(id) => id,
        None => {
            return NotFoundPage::new(
                &state.tera,
                Some("An item id must be provided for the \"/item\" endpoint."),
            )
            .render();
        }
    };

    let item_id = match item_id_unparsed.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return NotFoundPage::new(&state.tera, Some("The item id must be an integer number."))
                .render();
        }
    };

    let item = match Item::read(&state.pg_pool, item_id).await {
        Some(item) => item,
        None => {
            return NotFoundPage::new(
                &state.tera,
                Some("The indicated item number/id could not be found in the database."),
            )
            .render();
        }
    };

    ItemPage::new(&state.tera, item).render()
}
