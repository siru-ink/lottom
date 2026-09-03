use crate::{
    AppState,
    db::list::List,
    extractor::auth::AuthenticatedUser,
    template::{InternalServerErrorPage, ListPage, NotFoundPage},
};
use axum::{
    extract::{Query, State},
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    user: AuthenticatedUser,
) -> Response {
    let user_default_list_id = user.inner().default_list();

    let list_id = match params.get("list_id") {
        Some(id) => id.parse::<i32>().unwrap_or_else(|_| user_default_list_id),
        None => user_default_list_id,
    };

    let selected_list = match List::read(&state.pg_pool, list_id).await {
        Some(list) => list,
        None => {
            return NotFoundPage::new(&state.tera, Some("Selected list id does not exist."))
                .render();
        }
    };

    let list_items = match selected_list.get_items(&state.pg_pool).await {
        Ok(items) => items,
        Err(_) => return InternalServerErrorPage::new(&state.tera).render(),
    };

    ListPage::new(&state.tera, list_items).render()
}
