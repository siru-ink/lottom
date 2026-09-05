use crate::{
    AppState,
    db::{item::Item, list::List},
    extractor::auth::AuthenticatedUser,
    template::{InternalServerErrorPage, ListPage, NotFoundPage},
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    user: AuthenticatedUser,
) -> Response {
    let user_default_list_id = user.inner().default_list();

    let list_id = match params.get("list_id") {
        Some(id) => id.parse::<i32>().unwrap_or_else(|_| user_default_list_id),
        None => user_default_list_id,
    };

    let selected_list = match List::read(&state.pg_pool, list_id).await {
        Some(list) => list,
        None => {
            return NotFoundPage::new(&state.tera, Some("Selected list id does not exist."))
                .render();
        }
    };

    let list_items = match selected_list.get_items(&state.pg_pool).await {
        Ok(items) => items,
        Err(_) => return InternalServerErrorPage::new(&state.tera).render(),
    };

    ListPage::new(&state.tera, list_items, list_id).render()
}

#[derive(Debug, Deserialize)]
pub struct BoughtListItemsForm {
    list_id: i32,
    item_ids: Vec<i32>,
}

pub async fn post_remove_bought_items_from_list(
    State(state): State<Arc<AppState>>,
    Form(form): Form<BoughtListItemsForm>,
) -> Response {
    for item_id in form.item_ids {
        if let Some(item) = Item::read(&state.pg_pool, item_id).await {
            let _ = Item::delete(&state.pg_pool, &item).await;
        }
    }
    Redirect::to(&format!("/list?list_id={}", form.list_id)).into_response()
}
