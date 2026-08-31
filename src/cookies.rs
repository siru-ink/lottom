use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

#[derive(Debug)]
pub enum CookieValue {
    SessionID(i32),
}

#[derive(Debug)]
pub enum CookieModificationError {
    CookieJarPasswordInaccessible,
}

impl std::error::Error for CookieModificationError {}

impl std::fmt::Display for CookieModificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            CookieModificationError::CookieJarPasswordInaccessible => {
                "cookie jar decryption key not accessible"
            }
        };

        write!(f, "{}", description)
    }
}

enum CookieModificationKind {
    Set,
    Delete,
}

pub fn set_cookie(
    cookie_type: CookieValue,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    modify_cookie(cookie_type, CookieModificationKind::Set, cookie_jar)
}

pub fn remove_cookie(
    cookie_type: CookieValue,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    modify_cookie(cookie_type, CookieModificationKind::Delete, cookie_jar)
}

fn modify_cookie(
    cookie_type: CookieValue,
    modification: CookieModificationKind,
    cookie_jar: Cookies,
) -> Result<(), CookieModificationError> {
    let cookie_encryption_key = match crate::COOKIEKEY.get() {
        Some(val) => val,
        None => return Err(CookieModificationError::CookieJarPasswordInaccessible),
    };

    let encrypted_cookie_jar = cookie_jar.private(cookie_encryption_key);

    let cookie = match cookie_type {
        CookieValue::SessionID(val) => Cookie::build(("session_id", val.to_string()))
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

#[derive(Debug)]
pub enum CookieKey {
    SessionID,
}

#[derive(Debug)]
pub enum CookieRetrievalError {
    CookieJarPasswordInaccessible,
    CookieNotInJar,
    CookieValueNotParseable,
}

impl std::error::Error for CookieRetrievalError {}

impl std::fmt::Display for CookieRetrievalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookieRetrievalError::CookieJarPasswordInaccessible => {
                write!(f, "cookie jar decryption key not accessible")
            }
            CookieRetrievalError::CookieNotInJar => {
                write!(f, "requested cookie not found in request/cookie jar")
            }
            CookieRetrievalError::CookieValueNotParseable => {
                write!(
                    f,
                    "cookie value not according to expected data type and content"
                )
            }
        }
    }
}

impl CookieKey {
    fn get_cookie_name(&self) -> &str {
        match self {
            Self::SessionID => "session_id",
        }
    }

    pub fn get(&self, cookie_jar: Cookies) -> Result<CookieValue, CookieRetrievalError> {
        let cookie_jar_password = match crate::COOKIEKEY.get() {
            Some(passwd) => passwd,
            None => return Err(CookieRetrievalError::CookieJarPasswordInaccessible),
        };

        let encrypted_cookie_jar = cookie_jar.private(cookie_jar_password);

        let cookie = match encrypted_cookie_jar.get(self.get_cookie_name()) {
            Some(cookie) => cookie,
            None => return Err(CookieRetrievalError::CookieNotInJar),
        };

        let parsed_cookie = match self {
            Self::SessionID => {
                let session_id = match cookie.value().parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Err(CookieRetrievalError::CookieValueNotParseable),
                };

                CookieValue::SessionID(session_id)
            }
        };

        Ok(parsed_cookie)
    }
}
