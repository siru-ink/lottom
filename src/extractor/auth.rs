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
use urlencoding;

#[derive(Debug)]
pub struct AuthenticatedUser {
    user: User,
}

impl AuthenticatedUser {
    pub fn inner(self) -> User {
        self.user
    }
}

type RedirectToPath = String;

#[derive(Debug)]
pub enum AuthenticatedUserExtractorError {
    MissingSessionIDCookie(RedirectToPath),
    ExtractingCookieJarFailed(RedirectToPath),
    AppStateRetrievalFailed(RedirectToPath),
    ReferencedSessionNotInDB(RedirectToPath),
    ReferencedUserNotInDB(RedirectToPath),
    HowDidThisHappen(RedirectToPath),
}

impl IntoResponse for AuthenticatedUserExtractorError {
    fn into_response(self) -> Response {
        let redirect_uri = match self {
            AuthenticatedUserExtractorError::MissingSessionIDCookie(uri) => uri,
            AuthenticatedUserExtractorError::ExtractingCookieJarFailed(uri) => uri,
            AuthenticatedUserExtractorError::AppStateRetrievalFailed(uri) => uri,
            AuthenticatedUserExtractorError::ReferencedSessionNotInDB(uri) => uri,
            AuthenticatedUserExtractorError::ReferencedUserNotInDB(uri) => uri,
            AuthenticatedUserExtractorError::HowDidThisHappen(uri) => uri,
        };
        match self {
            _ => Redirect::to(&format!("/auth/login?forward_to={}", redirect_uri)).into_response(),
        }
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    State<Arc<AppState>>: FromRequestParts<S>,
{
    type Rejection = AuthenticatedUserExtractorError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uri = urlencoding::encode(&parts.uri.to_string()).into_owned();

        let cookie_jar = match Cookies::from_request_parts(parts, state).await {
            Ok(val) => val,
            Err(_) => {
                return Err(AuthenticatedUserExtractorError::ExtractingCookieJarFailed(
                    uri,
                ));
            }
        };

        let session_id_cookie = match crate::cookies::CookieKey::SessionID.get(cookie_jar) {
            Ok(cookie) => cookie,
            Err(_) => return Err(AuthenticatedUserExtractorError::MissingSessionIDCookie(uri)),
        };

        let session_id = match session_id_cookie {
            CookieValue::SessionID(id) => id,
            _ => return Err(AuthenticatedUserExtractorError::HowDidThisHappen(uri)),
        };

        let State(appstate): State<Arc<AppState>> =
            match State::from_request_parts(parts, state).await {
                Ok(val) => val,
                Err(_) => {
                    return Err(AuthenticatedUserExtractorError::AppStateRetrievalFailed(
                        uri,
                    ));
                }
            };

        let session = match Session::read(&appstate.pg_pool, session_id).await {
            Some(session) => session,
            None => {
                return Err(AuthenticatedUserExtractorError::ReferencedSessionNotInDB(
                    uri,
                ));
            }
        };

        let user = match session.get_user(&appstate.pg_pool).await {
            Some(user) => user,
            None => return Err(AuthenticatedUserExtractorError::ReferencedUserNotInDB(uri)),
        };

        Ok(AuthenticatedUser { user: user })
    }
}
