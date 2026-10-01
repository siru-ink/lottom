use crate::{
    AppState,
    db::{item::Item, list::List, prefill_item::PrefillItem},
    extractor::{auth::AuthenticatedUser, flash::Flash},
    template::{AddItemPage, InternalServerErrorPage, ItemAddImagePage, ItemPage, NotFoundPage},
};
use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{Form, Multipart};
use mime_guess::from_path;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

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
    flash: Flash,
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

    AddItemPage::new(&state.tera, prefill_items, list_id.to_owned(), flash.get()).render()
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

pub async fn get_add_img(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    _user: AuthenticatedUser,
    flash: Flash,
) -> Response {
    let raw_item_id = match params.get("item_id") {
        Some(id) => id,
        None => {
            flash.set("Error in url. To add image an item_id must be referenced in the url.");
            return Redirect::to("/").into_response();
        }
    };

    let item_id = match raw_item_id.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            flash
                .set("Error in url. The provided item id could not be parsed into a number <i32>.");
            return Redirect::to("/").into_response();
        }
    };

    let item = match Item::read(&state.pg_pool, item_id).await {
        Some(item) => item,
        None => {
            flash.set("The provided item id did not reference an existing shopping list item.");
            return Redirect::to("/").into_response();
        }
    };

    ItemAddImagePage::show(&state.tera, &item)
}

pub async fn post_add_img(
    State(state): State<Arc<AppState>>,
    flash: Flash,
    _user: AuthenticatedUser,
    mut files: Multipart,
) -> Response {
    let mut saved_name: Option<String> = None;
    let mut item_id: Option<String> = None;

    while let Ok(Some(mut field)) = files.next_field().await {
        let field_name = field.name().unwrap_or("").to_owned();

        if field_name == "item_id" {
            match field.text().await {
                Ok(value) => item_id = Some(value),
                Err(_) => {
                    flash.set("Could not read item id.");
                    return Redirect::to("/prefill/add").into_response();
                }
            }
            continue;
        }

        if field_name == "item_image" {
            let raw_file_name = match field.file_name() {
                Some(name) if !name.is_empty() => name.to_owned(),
                _ => {
                    while let Ok(Some(_)) = field.chunk().await {}
                    continue;
                }
            };

            let extension = std::path::Path::new(&raw_file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let uuid = Uuid::new_v4();
            let file_name = if extension.is_empty() {
                uuid.to_string()
            } else {
                format!("{uuid}.{extension}")
            };

            let save_path = PathBuf::from("uploads/").join(&file_name);

            let mut server_file = match File::create(&save_path).await {
                Ok(f) => f,
                Err(_) => {
                    flash.set("Could not create new file on server.");
                    return Redirect::to("/").into_response();
                }
            };

            loop {
                match field.chunk().await {
                    Ok(Some(chunk)) => {
                        if server_file.write_all(&chunk).await.is_err() {
                            let _ = fs::remove_file(&save_path).await;
                            flash.set("Error writing to file on server.");
                            return Redirect::to("/").into_response();
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        let _ = fs::remove_file(&save_path).await;
                        flash.set("Error writing to file on server.");
                        return Redirect::to("/").into_response();
                    }
                }
            }

            if server_file.flush().await.is_err() {
                let _ = fs::remove_file(&save_path).await;
                flash.set("Error writing to file on server.");
                return Redirect::to("/").into_response();
            }
            drop(server_file);
            saved_name = Some(file_name);
            continue;
        }

        // Drain any unknown fields so the stream doesn't get stuck
        while let Ok(Some(_)) = field.chunk().await {}
    }

    let Some(item_id) = item_id else {
        flash.set("Missing item id.");
        return Redirect::to("/prefill/add").into_response();
    };
    let Some(file_name) = saved_name else {
        flash.set("You must upload an item image file here.");
        return Redirect::to("/prefill/add").into_response();
    };

    let item_id_num = match item_id.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            flash.set("The item id must be a valid number.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let item = match Item::read(&state.pg_pool, item_id_num).await {
        Some(item) => item,
        None => {
            flash.set("The provided item id does not exist.");
            return Redirect::to("/prefill/add").into_response();
        }
    };

    let item = item.set_img_path(file_name).await;

    match Item::update(&state.pg_pool, &item).await {
        Some(_) => {
            flash.set("File saved successfully");
            return Redirect::to("/").into_response();
        }
        None => {
            flash.set("Something went wrong.");
            return Redirect::to("/").into_response();
        }
    }
}

pub async fn get_img(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    flash: Flash,
    _user: AuthenticatedUser,
) -> Response {
    let item_id_unparsed = match params.get("item_id") {
        Some(id) => id,
        None => {
            flash.set("Item picture could not be found.");
            return Redirect::to("/").into_response();
        }
    };

    let item_id = match item_id_unparsed.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            flash.set("Item id could not be parsed as a number.");
            return Redirect::to("/").into_response();
        }
    };

    let image_path = match Item::get_img_path_by_id(&state.pg_pool, item_id).await {
        Some(path) => path,
        None => {
            flash.set("Item does not have an associated image.");
            return Redirect::to("/").into_response();
        }
    };

    // Send the image data back to client by reading the file at /uploads/<image_path>
    let full_path = format!("uploads/{}", image_path);

    let mut file = match File::open(&full_path).await {
        Ok(file) => file,
        Err(_) => {
            flash.set("Item picture could not be found on disk.");
            return Redirect::to("/").into_response();
        }
    };

    // Read the first chunk to sniff the format.
    let mut header = [0u8; 512];
    let n = match file.read(&mut header).await {
        Ok(n) => n,
        Err(_) => {
            flash.set("Item picture could not be read.");
            return Redirect::to("/").into_response();
        }
    };

    // Identify the image type from magic bytes; fall back to extension.
    let mime_type = infer::get(&header[..n])
        .map(|kind| kind.mime_type())
        .and_then(|s| s.parse::<mime::Mime>().ok())
        .or_else(|| {
            from_path(&full_path)
                .first()
                .filter(|m| m.type_() == mime::IMAGE)
        });

    let mime_type = match mime_type {
        Some(m) => m,
        None => {
            flash.set("File is not a recognized image.");
            return Redirect::to("/").into_response();
        }
    };

    // Re-include the sniffed bytes, then stream the remainder of the file.
    let prefix = Bytes::copy_from_slice(&header[..n]);
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(tokio_stream::once(Ok(prefix)).chain(stream));

    let headers = [(header::CONTENT_TYPE, mime_type.as_ref())];
    (headers, body).into_response()
}
