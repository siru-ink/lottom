use axum::{middleware::from_fn, serve};
use sqlx::postgres::PgPool;
use std::sync::{Arc, OnceLock};
use tera::Tera;
use tokio::{main, net::TcpListener};
use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer, Key};

mod auth;
mod cookies;
mod db;
mod env;
mod extractor;
mod logging;
mod routing;

struct AppState {
    pg_pool: PgPool,
    tera: Tera,
}

static COOKIEKEY: OnceLock<Key> = OnceLock::new();

#[main]
async fn main() -> () {
    let config_values = match env::EnvConfig::setup() {
        Ok(val) => val,
        Err(e) => {
            println!("Error in provided environmental variables: {}", e);
            return;
        }
    };

    if let Err(_) = COOKIEKEY.set(Key::from(&config_values.cookiekey)) {
        println!("Error initializing encrypted cookies key.");
        return;
    }

    let pool = match db::init_db_connection(&config_values.postgres_connection_string).await {
        Some(pool) => pool,
        None => {
            println!("Database init failed.");
            return;
        }
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
