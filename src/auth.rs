use crate::db::session::Session;
use crate::db::user::User;
use crate::{AppState, cookies::CookieValue};
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;
use tower_cookies::Cookies;

#[derive(Debug)]
pub struct AuthenticatedUser {
    user: User,
}

impl AuthenticatedUser {
    pub fn get(&self) -> &User {
        &self.user
    }
}

#[derive(Debug)]
pub enum AuthenticationError {
    MissingSessionIDCookie,
    ExtractingCookieJarFailed,
    AppStateRetrievalFailed,
    ReferencedSessionNotInDB,
    ReferencedUserNotInDB,
    HowDidThisHappen,
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> Response {
        match self {
            _ => Redirect::to("/auth/login").into_response(),
        }
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    State<Arc<AppState>>: FromRequestParts<S>,
{
    type Rejection = AuthenticationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar = match Cookies::from_request_parts(parts, state).await {
            Ok(val) => val,
            Err(_) => return Err(AuthenticationError::ExtractingCookieJarFailed),
        };

        let session_id_cookie = match crate::cookies::CookieKey::SessionID.get(cookie_jar) {
            Ok(cookie) => cookie,
            Err(_) => return Err(AuthenticationError::MissingSessionIDCookie),
        };

        let session_id = match session_id_cookie {
            CookieValue::SessionID(id) => id,
            _ => return Err(AuthenticationError::HowDidThisHappen),
        };

        let State(appstate): State<Arc<AppState>> =
            match State::from_request_parts(parts, state).await {
                Ok(val) => val,
                Err(_) => return Err(AuthenticationError::AppStateRetrievalFailed),
            };

        let session = match Session::read(&appstate.pg_pool, session_id).await {
            Some(session) => session,
            None => return Err(AuthenticationError::ReferencedSessionNotInDB),
        };

        let user = match session.get_user(&appstate.pg_pool).await {
            Some(user) => user,
            None => return Err(AuthenticationError::ReferencedUserNotInDB),
        };

        Ok(AuthenticatedUser { user: user })
    }
}
