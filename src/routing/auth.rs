use crate::AppState;
use crate::extractor::auth::AuthenticatedUser;
use crate::extractor::flash::Flash;
use crate::routing::html_error;
use axum::extract::Query;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Context;

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

// #[derive(Deserialize)]
// struct LoginForm {
//     user_id: String,
//     password: String,
// }

// async fn post_login(
//     State(appstate): State<Arc<AppState>>,
//     cookies: Cookies,
//     Form(login_form): Form<LoginForm>,
// ) -> Response {
//     // Check whether the provided user id is in fact a number
//     let user_id: i32 = match login_form.user_id.parse() {
//         Ok(number) => number,
//         Err(_) => return Redirect::to("/auth/login").into_response(),
//     };

//     let user = match db::user::User::read(&appstate.pg_pool, user_id).await {
//         Some(user) => user,
//         None => return Redirect::to("/auth/login"),
//     };

//     let _ = match user.check_password(&login_form.password) {
//         CheckPasswordResult::PasswordCorrect => (),
//         CheckPasswordResult::PasswordWrong => return Redirect::to("/auth/login"),
//     };

//     // Create a new session
//     let new_session = match db::session::Session::create(&appstate.pg_pool, &user).await {
//         Some(new_session) => new_session,
//         None => return Redirect::to("/auth/login"),
//     };

//     // Store the session id as a private cookie
//     match set_cookie(new_session.as_cookie_value(), cookies) {
//         Ok(_) => return Redirect::to("/"),
//         Err(e) => return Redirect::to("/auth/login"),
//     }
// }
