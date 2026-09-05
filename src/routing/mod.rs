use crate::{
    AppState,
    extractor::auth::AuthenticatedUser,
    template::{IndexPage, InternalServerErrorPage},
};
use axum::{
    Router,
    extract::State,
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;

mod auth;
mod item;
mod list;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", get(auth::get_login).post(auth::post_login))
        .route("/auth/logout", get(auth::get_logout))
        .route("/list", get(list::get_list))
        .route("/item", get(item::get_item))
        .route("/item/modify", post(item::post_modify_item))
        .route("/", get(index))
}

async fn index(State(state): State<Arc<AppState>>, user: AuthenticatedUser) -> Response {
    let user = user.inner();

    let user_lists = match user.lists(&state.pg_pool).await {
        Ok(lists) => lists,
        Err(_) => return InternalServerErrorPage::new(&state.tera).render(),
    };

    IndexPage::new(&state.tera, user.name(), user_lists).render()
}
