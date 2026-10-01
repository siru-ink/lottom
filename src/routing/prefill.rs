use crate::{
    AppState,
    db::prefill_item::{PartialPrefillItem, PrefillItem},
    extractor::{auth::AuthenticatedUser, flash::Flash},
    template::{PrefillItemAddPage, PrefillItemNewPage},
};
use axum::{
    extract::{Query, State},
    http::{HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{Form, Multipart};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

pub async fn get_add(State(state): State<Arc<AppState>>, _user: AuthenticatedUser) -> Response {
    PrefillItemAddPage::show(&state.tera)
}

pub async fn post_add(
    State(state): State<Arc<AppState>>,
    flash: Flash,
    mut files: Multipart,
) -> Response {
    let optional_file = match files.next_field().await {
        Ok(option) => option,
        Err(_) => {
            flash.set("You must upload a prefill items file here.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let file = match optional_file {
        Some(file) => file,
        None => {
            flash.set("You must upload a prefill items file here.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let contents = match file.bytes().await {
        Ok(data) => data,
        Err(_) => {
            flash.set(
                "The server encountered an error reading the uploaded file. Please try again.",
            );
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let text = match std::str::from_utf8(&contents) {
        Ok(text) => text,
        Err(_) => {
            flash.set("Please upload a text file.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let parsed_content = match parse_prefill_file(text) {
        Some(content) => content,
        None => {
            flash.set("Please check the correct formatting of the prefill items text file.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    match PrefillItem::update_items(&state.pg_pool, parsed_content).await {
        Ok(_) => {
            flash.set("Prefill items updated successfully.");

            Redirect::to("/").into_response()
        }
        Err(_) => {
            flash.set("Error encountered applying database upsert.");
            return Redirect::to("/prefill/add").into_response();
        }
    }
}

fn parse_prefill_file(text: &str) -> Option<Vec<PartialPrefillItem>> {
    let mut items: Vec<PartialPrefillItem> = Vec::new();

    for line in text.lines() {
        if line.starts_with("#") {
            continue;
        }
        if line.is_empty() {
            continue;
        }

        let cols: Vec<&str> = line.split("|").map(|col| col.trim()).collect();

        if cols.len() != 5 {
            eprintln!("The file included a line with the incorrect column count.");
            return None;
        }

        let max_price = match cols[3].parse::<i32>() {
            Ok(num) => num,
            Err(_) => {
                eprintln!("The file included a line with a non-numeric max price.");
                return None;
            }
        };

        let min_price = match cols[4].parse::<i32>() {
            Ok(num) => num,
            Err(_) => {
                eprintln!("The file included a line with a non-numeric min price.");
                return None;
            }
        };

        let item = PartialPrefillItem {
            en_name: cols[0].to_string(),
            zh_name: cols[1].to_string(),
            de_name: cols[2].to_string(),
            cents_price_max: max_price,
            cents_price_min: min_price,
        };

        items.push(item);
    }

    Some(items)
}

pub async fn get_new(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let raw_list_id = match params.get("list_id") {
        Some(id) => id,
        None => {
            eprintln!("ERR: New prefill item route called without list id parameter.",);
            return Redirect::to("/").into_response();
        }
    };

    let list_id = match raw_list_id.parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            eprintln!(
                "ERR: List id passed to new prefill item route was not a number: {}",
                e
            );
            return Redirect::to("/").into_response();
        }
    };

    PrefillItemNewPage::show(&state.tera, list_id)
}

#[derive(Deserialize)]
pub struct CreatePrefillItemForm {
    en_name: String,
    zh_name: String,
    de_name: String,
}

pub async fn post_new(
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    flash: Flash,
    Query(params): Query<HashMap<String, String>>,
    Form(form): Form<CreatePrefillItemForm>,
) -> Response {
    let item = PartialPrefillItem::new(form.en_name, form.zh_name, form.de_name);
    let items = vec![item]; // to reuse existing update fn

    let raw_list_id = match params.get("list_id") {
        Some(id) => id,
        None => {
            eprintln!("ERR: New prefill item post route called without list id param.");
            flash.set("New prefill item route must be called with list_id parameter.");
            return Redirect::to("/").into_response();
        }
    };

    let list_id = match raw_list_id.parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            eprintln!(
                "ERR: New prefill item post route called with non-numeric list id param: {}",
                e
            );
            flash.set("New prefill item route must be called with a numeric list_id parameter.");
            return Redirect::to("/").into_response();
        }
    };

    match PrefillItem::update_items(&state.pg_pool, items).await {
        Ok(_) => {
            flash.set("New prefill item created successfully.");
            Redirect::to(&format!("/item/add?list_id={}", list_id)).into_response()
        }
        Err(e) => {
            eprintln!("ERR: Failed to create new prefill item: {}", e);
            flash.set("Creating new prefill item failed. Please try again.");
            Redirect::to(&format!("/prefill/new?list_id={}", list_id)).into_response()
        }
    }
}

pub async fn get_export(State(state): State<Arc<AppState>>, flash: Flash) -> Response {
    let prefill_items = match PrefillItem::get_all(&state.pg_pool).await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("ERR: Failed to retrieve all prefill items: {}", e);
            flash.set("Failed to retrive all prefill items.");
            return Redirect::to("/").into_response();
        }
    };

    let content = prefill_items
        .iter()
        .map(|item| item.file_repr())
        .collect::<Vec<_>>()
        .join("\n");

    let mut response = content.into_response();
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"export.txt\""),
    );

    response
}
