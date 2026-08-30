use crate::AppState;
use crate::cookies::{CookieType, remove_cookie, set_cookie};
use crate::db::PasswordCheckResult;
use axum::{
    Form, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use sqlx::{PgPool, query};
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
    match set_cookie(
        CookieType::SessionID(new_session.get_cookie_reference()),
        cookies,
    ) {
        Ok(_) => _,
        Err(e) => return Redirect::to("/auth/login"),
    };

    // Redirect to index
    Redirect::to("/")
}

async fn get_logout(State(appstate): State<Arc<AppState>>, cookies: Cookies) -> Response {
    let description = match remove_cookie(CookieType::SessionID(0), cookies) {
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

// pub async fn check_authorization(cookies: &Cookies, pool: &PgPool) -> bool {
//     let encrypted_cookie_jar = cookies.private(crate::COOKIEKEY.get().unwrap());
//     let session_id_cookie = match encrypted_cookie_jar.get("session_id") {
//         Some(session_id_cookie) => session_id_cookie,
//         None => return false,
//     };
//     let session_id = match session_id_cookie.value().parse::<i32>() {
//         Ok(number) => number,
//         Err(_) => {
//             encrypted_cookie_jar.remove(
//                 Cookie::build(("session_id", ""))
//                     .domain("localhost")
//                     .path("/")
//                     .max_age(Duration::days(7))
//                     .secure(false)
//                     .http_only(true)
//                     .same_site(SameSite::Strict)
//                     .build(),
//             );
//             return false;
//         }
//     };
//     match query!("SELECT * FROM sessions WHERE id = $1", session_id)
//         .fetch_optional(pool)
//         .await
//     {
//         Ok(_) => true,
//         Err(_) => {
//             encrypted_cookie_jar.remove(
//                 Cookie::build(("session_id", ""))
//                     .domain("localhost")
//                     .path("/")
//                     .max_age(Duration::days(7))
//                     .secure(false)
//                     .http_only(true)
//                     .same_site(SameSite::Strict)
//                     .build(),
//             );
//             false
//         }
//     }
// }

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", get(get_logout))
}
