use crate::AppState;
use crate::cookies::{CookieValue, remove_cookie, set_cookie};
use crate::db;
use crate::db::user::CheckPasswordResult;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;
use tera::context;
use tower_cookies::Cookies;

#[derive(Deserialize)]
struct LoginForm {
    user_id: String,
    password: String,
}

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
async fn post_login(
    State(appstate): State<Arc<AppState>>,
    cookies: Cookies,
    Form(login_form): Form<LoginForm>,
) -> impl IntoResponse {
    let user_id: i32 = match login_form.user_id.parse() {
        Ok(number) => number,
        Err(_) => return Redirect::to("/auth/login"),
    };

    let user = match db::user::User::read(&appstate.pg_pool, user_id).await {
        Some(user) => user,
        None => return Redirect::to("/auth/login"),
    };

    let _ = match user.check_password(&login_form.password) {
        CheckPasswordResult::PasswordCorrect => (),
        CheckPasswordResult::PasswordWrong => return Redirect::to("/auth/login"),
    };

    // Create a new session
    let new_session = match db::session::Session::create(&appstate.pg_pool, &user).await {
        Some(new_session) => new_session,
        None => return Redirect::to("/auth/login"),
    };

    // Store the session id as a private cookie
    match set_cookie(new_session.as_cookie_value(), cookies) {
        Ok(_) => return Redirect::to("/"),
        Err(e) => return Redirect::to("/auth/login"),
    }
}

async fn get_logout(State(appstate): State<Arc<AppState>>, cookies: Cookies) -> Response {
    let description = match remove_cookie(CookieValue::SessionID(0), cookies) {
        Ok(_) => "Finished logging out. Have a nice day.",
        Err(_) => "Logging out failed. Please try again.",
    };

    let context = context! {
        message => description
    };
    match appstate.tera.render("logout.html", &context) {
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

async fn index(_auth_user: crate::auth::AuthenticatedUser) -> Response {
    "Hello, world!".into_response()
}

pub fn get_routes() -> Router<Arc<AppState>> {
    let auth_router = Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", get(get_logout));

    Router::new()
        .route("/", get(index))
        .nest("/auth", auth_router)
}
