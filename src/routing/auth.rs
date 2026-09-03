use crate::AppState;
use crate::cookies::{CookieValue, remove_cookie, set_cookie};
use crate::db::{
    session::Session,
    user::{CheckPasswordResult, User},
};
use crate::extractor::flash::Flash;
use crate::template::{LoginPage, LogoutPage};
use axum::extract::Query;
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower_cookies::Cookies;

pub async fn get_login(
    State(appstate): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    flash: Flash,
) -> Response {
    let forward_to = params.get("forward_to");
    let flash_message = flash.get();
    LoginPage::new(&appstate.tera, forward_to, flash_message).render()
}

#[derive(Deserialize)]
pub struct LoginForm {
    user_id: String,
    password: String,
    forward_to: String,
}

pub async fn post_login(
    State(appstate): State<Arc<AppState>>,
    cookies: Cookies,
    flash: Flash,
    Form(login_form): Form<LoginForm>,
) -> Response {
    let forward_to_urlencoded = urlencoding::encode(&login_form.forward_to).to_owned();

    let user_id: i32 = match login_form.user_id.parse() {
        Ok(number) => number,
        Err(_) => {
            flash.set("Please enter a numerical user id to log in.");
            return Redirect::to(&format!("/auth/login?forward_to={}", forward_to_urlencoded))
                .into_response();
        }
    };

    let user = match User::read(&appstate.pg_pool, user_id).await {
        Some(user) => user,
        None => {
            flash.set("The provided user id does not exist.");
            return Redirect::to(&format!("/auth/login?forward_to={}", forward_to_urlencoded))
                .into_response();
        }
    };

    let _ = match user.check_password(&login_form.password) {
        CheckPasswordResult::PasswordCorrect => (),
        CheckPasswordResult::PasswordWrong => {
            flash.set("The provided password was not correct.");
            return Redirect::to(&format!("/auth/login?forward_to={}", forward_to_urlencoded))
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
            return Redirect::to(&format!("/auth/login?forward_to={}", forward_to_urlencoded))
                .into_response();
        }
    };

    // Store the session id as a private cookie
    match set_cookie(new_session.as_cookie_value(), cookies) {
        Ok(_) => return Redirect::to(&login_form.forward_to).into_response(),
        Err(_) => {
            flash.set("Please allow cookies for this domain/website to be able to log in.");
            return Redirect::to(&format!("/auth/login?forward_to={}", forward_to_urlencoded))
                .into_response();
        }
    }
}

pub async fn get_logout(State(state): State<Arc<AppState>>, cookie_jar: Cookies) -> Response {
    let _ = remove_cookie(CookieValue::SessionID(0), cookie_jar);
    LogoutPage::new(&state.tera, "You have been logged out. Have a nice day.").render()
}
