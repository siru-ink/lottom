use crate::AppState;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use std::sync::Arc;
use tera::context;

async fn get_login(State(appstate): State<Arc<AppState>>) -> impl IntoResponse {
    let context = context! {};
    match appstate.tera.render("login.html", &context) {
        Ok(val) => Html(val).into_response(),
        Err(_) => {
            println!("Error rendering the login.html template.");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error rendering template.",
            )
                .into_response();
        }
    }
}

#[axum::debug_handler]
async fn post_login() -> &'static str {
    "This is the post login page."
}

async fn get_logout() -> &'static str {
    "This is the logout page."
}

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", get(get_logout))
}
