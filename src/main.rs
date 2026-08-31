use axum::middleware::from_fn;
use axum::serve;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env::var;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::OnceLock;
use tera::Tera;
use tokio::{main, net::TcpListener};
use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer, Key};

mod auth;
mod cookies;
mod db;
mod logging;
mod routing;

struct AppState {
    pg_pool: PgPool,
    tera: Tera,
}

static COOKIEKEY: OnceLock<Key> = OnceLock::new();

#[main]
async fn main() -> () {
    let config_values = match get_env() {
        Ok(val) => val,
        Err(e) => {
            println!("Error in provided environmental variables: {}", e);
            return;
        }
    };

    let key_bytes = match BASE64.decode(config_values.cookiekey) {
        Ok(val) => val,
        Err(e) => {
            println!(
                "Error decoding the COOKIEKEY env variable from base64: {}",
                e
            );
            return;
        }
    };
    if let Err(_) = COOKIEKEY.set(Key::from(&key_bytes)) {
        println!("Error initializing encrypted cookies key.");
        return;
    }

    let postgres_connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        config_values.postgres_username,
        config_values.postgres_password,
        config_values.postgres_address,
        config_values.postgres_port,
        config_values.postgres_database_name
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_connection_string)
        .await
        .expect("Database should be reachable.");

    if let Err(e) = db::apply_db_migrations(&pool).await {
        println!("Error appliying database migrations: {}", e);
        return;
    };

    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")
        .expect("Tera should be able to load templates from <templates/>.");

    let appstate = Arc::new(AppState {
        pg_pool: pool,
        tera: tera,
    });

    let app = routing::get_routes()
        .layer(
            ServiceBuilder::new()
                .layer(from_fn(logging::logger))
                .layer(CookieManagerLayer::new()),
        )
        .with_state(appstate);

    let listener = TcpListener::bind("0.0.0.0:8150").await.unwrap();

    serve(listener, app).await.unwrap();
}

struct EnvConfig {
    postgres_username: String,
    postgres_password: String,
    postgres_address: String,
    postgres_port: String,
    postgres_database_name: String,
    cookiekey: String,
}

#[derive(Debug)]
enum EnvConfigError {
    MissingPostgresUsername,
    MissingPostgresPassword,
    MissingPostgresAddress,
    MissingPostgresPort,
    MissingPostgresDatabaseName,
    MissingCookieKeyValue,
}

impl Display for EnvConfigError {
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
        }
    }
}

impl Error for EnvConfigError {}

fn get_env() -> Result<EnvConfig, EnvConfigError> {
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
    let cookiekey = match var("COOKIEKEY") {
        Ok(val) => val,
        Err(_) => return Err(EnvConfigError::MissingCookieKeyValue),
    };

    Ok(EnvConfig {
        postgres_username: postgres_username,
        postgres_password: postgres_password,
        postgres_address: postgres_address,
        postgres_port: postgres_port,
        postgres_database_name: postgres_database_name,
        cookiekey: cookiekey,
    })
}
