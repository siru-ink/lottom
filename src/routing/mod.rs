use crate::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

mod auth;
mod html_error;

pub fn get_routes() -> Router<Arc<AppState>> {
    let auth_router = Router::new().route("/login", get(auth::get_login));

    Router::new().nest("/auth", auth_router)
}
