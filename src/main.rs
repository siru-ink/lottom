use axum::extract::Request;
use axum::middleware::{Next, from_fn};
use axum::response::Response;
use axum::routing::get;
use axum::{Router, serve};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env::var;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use tera::Tera;
use tokio::{main, net::TcpListener};
use tower::ServiceBuilder;

mod auth;
mod db;

struct AppState {
    _pg_pool: PgPool,
    tera: Tera,
}

#[main]
async fn main() -> () {
    let config_values = match get_env() {
        Ok(val) => val,
        Err(e) => {
            println!("Error in provided environmental variables: {}", e);
            return;
        }
    };

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
        _pg_pool: pool,
        tera: tera,
    });

    let app = Router::new()
        .route("/", get(index))
        .nest("/auth", auth::get_routes())
        .layer(ServiceBuilder::new().layer(from_fn(logger)))
        .with_state(appstate);

    let listener = TcpListener::bind("0.0.0.0:11000").await.unwrap();

    serve(listener, app).await.unwrap();
}

async fn logger(request: Request, next: Next) -> Response {
    println!("Serving {}", request.uri().to_string());
    next.run(request).await
}

struct EnvConfig {
    postgres_username: String,
    postgres_password: String,
    postgres_address: String,
    postgres_port: String,
    postgres_database_name: String,
}

#[derive(Debug)]
enum EnvConfigError {
    MissingPostgresUsername,
    MissingPostgresPassword,
    MissingPostgresAddress,
    MissingPostgresPort,
    MissingPostgresDatabaseName,
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

    Ok(EnvConfig {
        postgres_username: postgres_username,
        postgres_password: postgres_password,
        postgres_address: postgres_address,
        postgres_port: postgres_port,
        postgres_database_name: postgres_database_name,
    })
}

async fn index() -> &'static str {
    "Hello, world!"
}
