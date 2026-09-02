use crate::AppState;
use crate::cookies::set_cookie;
use crate::db::{
    session::Session,
    user::{CheckPasswordResult, User},
};
use crate::extractor::auth::AuthenticatedUser;
use crate::extractor::flash::Flash;
use crate::routing::html_error;
use axum::extract::Query;
use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Context;
use tower_cookies::Cookies;

pub async fn get_login(
    State(appstate): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    flash: Flash,
) -> Response {
    let mut context = Context::new();

    let default_forward_path = "/".to_string();
    let forward_to = params.get("forward_to").unwrap_or(&default_forward_path);

    context.insert("forward_to", forward_to);

    if let Some(text) = flash.get() {
        context.insert("flash_message", &text);
    }

    match appstate.tera.render("login.html", &context) {
        Ok(val) => Html(val).into_response(),
        Err(e) => {
            return html_error::not_found(
                appstate,
                &format!("Login template could not be found: {}", e),
            );
        }
    }
}

pub async fn set_flash(flash: Flash) -> Response {
    flash.set("Testing, 1 2 3 , testing");
    Redirect::to("/auth/login").into_response()
}

pub async fn test_auth(_user: AuthenticatedUser) -> Response {
    "You are now logged in.".into_response()
}

#[derive(Deserialize)]
struct LoginForm {
    user_id: String,
    password: String,
    forward_to: String,
}

async fn post_login(
    State(appstate): State<Arc<AppState>>,
    cookies: Cookies,
    Form(login_form): Form<LoginForm>,
    flash: Flash,
) -> Response {
    let redirect_uri = urlencoding::encode(&login_form.forward_to).to_owned();

    // Check whether the provided user id is in fact a number
    let user_id: i32 = match login_form.user_id.parse() {
        Ok(number) => number,
        Err(_) => {
            flash.set("Please enter a numerical user id to log in.");
            return Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri))
                .into_response();
        }
    };

    let user = match User::read(&appstate.pg_pool, user_id).await {
        Some(user) => user,
        None => {
            flash.set("The provided user id does not exist.");
            return Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri))
                .into_response();
        }
    };

    let _ = match user.check_password(&login_form.password) {
        CheckPasswordResult::PasswordCorrect => (),
        CheckPasswordResult::PasswordWrong => {
            flash.set("The provided password was not correct.");
            return Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri))
                .into_response();
        }
    };

    // Create a new session
    let new_session = match Session::create(&appstate.pg_pool, &user).await {
        Some(new_session) => new_session,
        None => {
            flash.set(
                "Logging in failed because no session id could be created. \
                Please try again or contact the system administrator.",
            );
            return Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri))
                .into_response();
        }
    };

    // Store the session id as a private cookie
    match set_cookie(new_session.as_cookie_value(), cookies) {
        Ok(_) => return Redirect::to(&redirect_uri).into_response(),
        Err(_) => {
            flash.set("Please allow cookies for this domain/website to be able to log in.");
            return Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri))
                .into_response();
        }
    }
}
