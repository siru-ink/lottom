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
mod user;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", get(auth::get_login).post(auth::post_login))
        .route("/auth/logout", get(auth::get_logout))
        .route("/list", get(list::get_list))
        .route(
            "/list/bought",
            post(list::post_remove_bought_items_from_list),
        )
        .route(
            "/list/modify",
            get(list::get_modify).post(list::post_modify),
        )
        .route("/list/share", get(list::get_share).post(list::post_share))
        .route("/item", get(item::get_display))
        .route("/item/modify", post(item::post_modify))
        .route("/item/add", get(item::get_add).post(item::post_add))
        .route(
            "/user/signup",
            get(user::get_signup).post(user::post_signup),
        )
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
