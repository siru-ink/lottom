use crate::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

mod auth;

pub fn get_routes() -> Router<Arc<AppState>> {
    let auth_router = Router::new()
        .route("/login", get(auth::get_login).post(auth::post_login))
        .route("/logout", get(auth::get_logout));

    Router::new().nest("/auth", auth_router)
}
