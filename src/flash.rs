use crate::cookies::{self, CookieKey, CookieRetrievalError, CookieValue, set_cookie};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use tower_cookies::Cookies;

#[derive(Debug)]
pub struct FlashMessage {
    pub message: String,
}

impl FlashMessage {
    pub fn new(message: String) -> FlashMessage {
        FlashMessage { message: message }
    }

    pub fn set(self, cookie_jar: Cookies) {
        let cookie_value = CookieValue::FlashMessage(self.message);
        let _ = set_cookie(cookie_value, cookie_jar);
    }
}

#[derive(Debug)]
pub enum FlashMessageError {
    CookieRetrievalError,
}

impl std::error::Error for FlashMessageError {}

impl std::fmt::Display for FlashMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "flash message error should never be called in code")
    }
}

impl IntoResponse for FlashMessageError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "flash messsage error should never be called",
        )
            .into_response()
    }
}

impl<S> OptionalFromRequestParts<S> for FlashMessage
where
    S: Send + Sync,
{
    type Rejection = FlashMessageError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let cookie_jar = match Cookies::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(_) => return Ok(None),
        };

        let flash_message = match CookieKey::FlashMessage.get(cookie_jar) {
            Ok(crate::cookies::CookieValue::FlashMessage(message)) => message,
            Ok(_) => return Ok(None),
            Err(CookieRetrievalError::CookieNotInJar) => return Ok(None),
            Err(_) => return Err(FlashMessageError::CookieRetrievalError),
        };

        Ok(Some(FlashMessage {
            message: flash_message,
        }))
    }
}
