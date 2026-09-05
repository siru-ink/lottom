use crate::{
    AppState,
    db::item::Item,
    template::{InternalServerErrorPage, ItemPage, NotFoundPage},
};
use axum::{
    Form,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
pub struct ModifyItemForm {
    id: i32,
    list_id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    img_path: String,
    estimated_euro_price: i32,
}

#[axum::debug_handler]
pub async fn post_modify_item(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ModifyItemForm>,
) -> Response {
    let img_path_parsed = if form.img_path.is_empty() {
        None
    } else {
        Some(form.img_path)
    };
    let changed_item = Item::new(
        form.id,
        form.list_id,
        form.en_name,
        form.zh_name,
        form.de_name,
        img_path_parsed,
        form.estimated_euro_price,
    );

    match Item::update(&state.pg_pool, &changed_item).await {
        Some(item) => Redirect::to(&format!("/item?item_id={}", item.id())).into_response(),
        None => InternalServerErrorPage::new(&state.tera).render(),
    }
}
