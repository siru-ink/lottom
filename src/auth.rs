use crate::db::PasswordCheckResult;
use crate::{AppState, db};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;
use std::sync::Arc;
use tera::context;
use tower_cookies::cookie::SameSite;
use tower_cookies::cookie::time::Duration;
use tower_cookies::{Cookie, Cookies};

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

    let user = match db::read_user(&appstate.pg_pool, user_id).await {
        Some(user) => user,
        None => return Redirect::to("/auth/login"),
    };

    let _ = match user.check_password(&login_form.password) {
        PasswordCheckResult::Valid => (),
        PasswordCheckResult::Invalid => return Redirect::to("/auth/login"),
    };

    // Create a new session
    let new_session = match db::create_session(&appstate.pg_pool, &user).await {
        Some(new_session) => new_session,
        None => return Redirect::to("/auth/login"),
    };
    // Store the session id as a private cookie
    let encrypted_cookie_jar = cookies.private(crate::COOKIEKEY.get().unwrap());
    encrypted_cookie_jar.add(
        Cookie::build(("session_id", new_session.get_cookie_reference().to_string()))
            .domain("localhost")
            .path("/")
            .max_age(Duration::days(7))
            .secure(false)
            .http_only(true)
            .same_site(SameSite::Strict)
            .build(),
    );
    // Redirect to index
    Redirect::to("/")
}

async fn get_logout() -> &'static str {
    "This is the logout page."
}

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", get(get_logout))
}
