use crate::AppState;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use tera::context;

pub fn not_found(state: Arc<AppState>, flash: &str) -> Response {
    let context = context! { flash => flash};
    match state.tera.render("not_found.html", &context) {
        Ok(html_source) => Html(html_source).into_response(),
        Err(_) => {
            println!("not_found.html HTML template could not be found");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error rendering template.",
            )
                .into_response();
        }
    }
}
