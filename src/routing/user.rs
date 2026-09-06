use crate::{
    AppState,
    db::user::User,
    extractor::flash::Flash,
    template::{InternalServerErrorPage, SignUpPage},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;
use std::sync::Arc;

pub async fn get_signup(State(state): State<Arc<AppState>>) -> Response {
    SignUpPage::new(&state.tera).render()
}

#[derive(Debug, Deserialize)]
pub struct SignUpForm {
    username: String,
    password: String,
}

pub async fn post_signup(
    State(state): State<Arc<AppState>>,
    flash: Flash,
    Form(form): Form<SignUpForm>,
) -> Response {
    let user_id = match User::create(&state.pg_pool, &form.username, &form.password).await {
        Some(user) => user,
        None => return InternalServerErrorPage::new(&state.tera).render(),
    };

    flash.set(&format!(
        "Your user id is: {}. Use this id to log in.",
        user_id
    ));

    Redirect::to("/auth/login").into_response()
}
