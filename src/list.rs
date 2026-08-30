use crate::AppState;
use crate::auth::check_authorization;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use std::collections::HashMap;
use std::sync::Arc;
use tower_cookies::Cookies;

async fn get_list(
    Query(params): Query<HashMap<String, String>>,
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Response {
    if !check_authorization(&cookies, &state.pg_pool).await {
        return Redirect::to("/auth/login").into_response();
    }

    let list_id = match params.get("list_id") {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    "Unfinished".into_response()
}

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().route("/list", get(get_list))
}
