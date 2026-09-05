use crate::{AppState, db::user::User, template::SignUpPage};
use axum::{extract::State, response::Response};
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
    Form(form): Form<SignUpForm>,
) -> Response {
    let _ = User::create(&state.pg_pool, &form.username, &form.password).await;

    todo!()
}
