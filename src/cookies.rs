use std::{error::Error, fmt::Display};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

pub enum CookieType {
    SessionID(i32),
}

#[derive(Debug)]
pub enum CookieModificationError {
    KeyRetrievalError,
}

impl Display for CookieModificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            CookieModificationError::KeyRetrievalError => {
                "application-wide, static, cookie-encryption key could not be accessed"
            }
        };

        write!(f, "{}", description)
    }
}

impl Error for CookieModificationError {}

enum CookieModificationKind {
    Set,
    Delete,
}

pub fn set_cookie(
    cookie_type: CookieType,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    modify_cookie(cookie_type, CookieModificationKind::Set, cookie_jar)
}

pub fn remove_cookie(
    cookie_type: CookieType,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    modify_cookie(cookie_type, CookieModificationKind::Delete, cookie_jar)
}

fn modify_cookie(
    cookie_type: CookieType,
    modification: CookieModificationKind,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    let cookie_encryption_key = match crate::COOKIEKEY.get() {
        Some(val) => val,
        None => return Err(CookieModificationError::KeyRetrievalError),
    };

    let encrypted_cookie_jar = cookie_jar.private(cookie_encryption_key);

    let cookie = match cookie_type {
        CookieType::SessionID(val) => Cookie::build(("session_id", val.to_string()))
            .domain("grocery.siru.ink")
            .path("/")
            .http_only(true)
            .max_age(Duration::days(7))
            .secure(true)
            .same_site(SameSite::Strict)
            .build(),
    };

    match modification {
        CookieModificationKind::Set => encrypted_cookie_jar.add(cookie),
        CookieModificationKind::Delete => encrypted_cookie_jar.remove(cookie),
    };

    Ok(())
}
