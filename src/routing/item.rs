use crate::{
    AppState,
    db::{item::Item, list::List, prefill_item::PrefillItem},
    extractor::auth::AuthenticatedUser,
    template::{AddItemPage, InternalServerErrorPage, ItemPage, NotFoundPage},
};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

pub async fn get_display(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
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

pub async fn post_modify(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
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
        Some(item) => Redirect::to(&format!("/list?list_id={}", item.list_id())).into_response(),
        None => InternalServerErrorPage::new(&state.tera).render(),
    }
}

pub async fn get_add(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    _user: AuthenticatedUser,
) -> Response {
    let prefill_items = match PrefillItem::get_all(&state.pg_pool).await {
        Ok(items) => items,
        Err(_) => return InternalServerErrorPage::new(&state.tera).render(),
    };

    let list_id = match params.get("list_id") {
        Some(id) => id,
        None => return InternalServerErrorPage::new(&state.tera).render(),
    };

    AddItemPage::new(&state.tera, prefill_items, list_id.to_owned()).render()
}

#[derive(Debug, Deserialize)]
pub struct AddItemForm {
    ids: Vec<i32>,
    list_id: i32,
}

pub async fn post_add(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    Form(form): Form<AddItemForm>,
) -> Response {
    println!("{:#?}", form);

    let list = match List::read(&state.pg_pool, form.list_id).await {
        Some(list) => list,
        None => return InternalServerErrorPage::new(&state.tera).render(),
    };

    for prefill_id in form.ids {
        if let Some(prefill_item) = PrefillItem::read(&state.pg_pool, prefill_id).await {
            let _ = Item::from_prefill_item(&state.pg_pool, list.get_id(), prefill_item).await;
        }
    }

    Redirect::to(&format!("/list?list_id={}", list.get_id())).into_response()
}
