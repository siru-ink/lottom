use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::env::var;

#[derive(Debug)]
pub struct EnvConfig {
    pub postgres_connection_string: String,
    pub cookiekey: Vec<u8>,
}

#[derive(Debug)]
pub enum EnvConfigError {
    MissingPostgresUsername,
    MissingPostgresPassword,
    MissingPostgresAddress,
    MissingPostgresPort,
    MissingPostgresDatabaseName,
    MissingCookieKeyValue,
    CookieKeyValueNotParseable,
}

impl std::error::Error for EnvConfigError {}

impl std::fmt::Display for EnvConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvConfigError::MissingPostgresUsername => {
                write!(f, "Missing POSTGRES_USERNAME env variable.")
            }
            EnvConfigError::MissingPostgresPassword => {
                write!(f, "Missing POSTGRES_PASSWORD env variable.")
            }
            EnvConfigError::MissingPostgresAddress => {
                write!(f, "Missing POSTGRES_PASSWORD env variable.")
            }
            EnvConfigError::MissingPostgresPort => {
                write!(f, "Missing POSTGRES_PORT env variable.")
            }
            EnvConfigError::MissingPostgresDatabaseName => {
                write!(f, "Missing POSTGRES_DATABASE_NAME env variable.")
            }
            EnvConfigError::MissingCookieKeyValue => {
                write!(f, "Missing COOKIEKEY env variable.")
            }
            EnvConfigError::CookieKeyValueNotParseable => {
                write!(f, "cookie key value is not base64 decodable")
            }
        }
    }
}

impl EnvConfig {
    pub fn setup() -> Result<EnvConfig, EnvConfigError> {
        let postgres_username = match var("POSTGRES_USERNAME") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingPostgresUsername),
        };
        let postgres_password = match var("POSTGRES_PASSWORD") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingPostgresPassword),
        };
        let postgres_address = match var("POSTGRES_ADDRESS") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingPostgresAddress),
        };
        let postgres_port = match var("POSTGRES_PORT") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingPostgresPort),
        };
        let postgres_database_name = match var("POSTGRES_DATABASE_NAME") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingPostgresDatabaseName),
        };

        let postgres_connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            postgres_username,
            postgres_password,
            postgres_address,
            postgres_port,
            postgres_database_name
        );

        let cookie_key = match var("COOKIEKEY") {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::MissingCookieKeyValue),
        };

        let cookie_key_bytes = match BASE64.decode(cookie_key) {
            Ok(val) => val,
            Err(_) => return Err(EnvConfigError::CookieKeyValueNotParseable),
        };

        Ok(EnvConfig {
            postgres_connection_string: postgres_connection_string,
            cookiekey: cookie_key_bytes,
        })
    }
}
