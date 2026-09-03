use crate::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

mod auth;
mod list;

pub fn get_routes() -> Router<Arc<AppState>> {
    let auth_router = Router::new()
        .route("/login", get(auth::get_login).post(auth::post_login))
        .route("/logout", get(auth::get_logout));

    let list_router = Router::new().route("/", get(list::get_list));

    Router::new()
        .nest("/auth", auth_router)
        .nest("/list", list_router)
}
