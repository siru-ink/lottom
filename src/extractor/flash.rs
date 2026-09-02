use crate::cookies::set_cookie;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use tower_cookies::Cookies;

use crate::cookies::{CookieKey, CookieValue};

#[derive(Debug)]
pub struct Flash {
    message: Option<String>,
    cookie_jar: Cookies,
}

impl Flash {
    fn new(message: Option<String>, cookie_jar: Cookies) -> Flash {
        Flash {
            message: message,
            cookie_jar: cookie_jar,
        }
    }

    pub fn set(self, message: &str) {
        let _ = set_cookie(
            CookieValue::FlashMessage(message.to_string()),
            self.cookie_jar,
        );
    }

    pub fn get(self) -> Option<String> {
        self.message
    }
}

#[derive(Debug)]
pub enum FlashExtractorError {
    CookieJarExtractorFailed,
    WrongCookieTypeRetrieved,
}

impl std::error::Error for FlashExtractorError {}

impl std::fmt::Display for FlashExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => write!(f, "could not extract flash"), // TODO improve this error message
        }
    }
}

impl IntoResponse for FlashExtractorError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "flash extraction failed").into_response()
    }
}

impl<S> FromRequestParts<S> for Flash
where
    S: Send + Sync,
{
    type Rejection = FlashExtractorError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar = match Cookies::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(_) => return Err(FlashExtractorError::CookieJarExtractorFailed),
        };

        let flash_cookie = match CookieKey::FlashMessage.get(cookie_jar.clone()) {
            Ok(cookie) => cookie,
            Err(_) => return Ok(Flash::new(None, cookie_jar)),
        };

        let flash_message = match flash_cookie {
            CookieValue::FlashMessage(message) => message,
            _ => return Err(FlashExtractorError::WrongCookieTypeRetrieved), // this should be unreachable
        };

        Ok(Flash::new(Some(flash_message), cookie_jar))
    }
}
