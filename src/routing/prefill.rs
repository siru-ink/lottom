use crate::{
    AppState,
    db::prefill_item::{PartialPrefillItem, PrefillItem},
    extractor::{auth::AuthenticatedUser, flash::Flash},
    template::PrefillItemAddPage,
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Multipart;
use std::sync::Arc;

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
        let cols: Vec<&str> = line.split("|").collect();

        if cols.len() != 5 {
            return None;
        }

        let max_price = match cols[3].parse::<i32>() {
            Ok(num) => num,
            Err(_) => return None,
        };

        let min_price = match cols[4].parse::<i32>() {
            Ok(num) => num,
            Err(_) => return None,
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
