use crate::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

mod auth;
mod list;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", get(auth::get_login).post(auth::post_login))
        .route("/auth/logout", get(auth::get_logout))
        .route("/list", get(list::get_list))
}
