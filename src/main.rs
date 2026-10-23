use axum::{middleware::from_fn, serve};
use sqlx::postgres::PgPool;
use std::sync::{Arc, OnceLock};
use tera::Tera;
use tokio::{main, net::TcpListener};
use tower_cookies::{CookieManagerLayer, Key};

mod cookies;
mod db;
mod env;
mod extractor;
mod routing;
mod template;

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
        .layer(CookieManagerLayer::new())
        .with_state(appstate);

    #[cfg(debug_assertions)]
    let app = app.layer(from_fn(logger));

    #[cfg(not(debug_assertions))]
    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .expect("Port 80 should allow binding a new listener");

    #[cfg(debug_assertions)]
    let listener = TcpListener::bind("127.0.0.1:8150")
        .await
        .expect("Port 8160 should allow binding a new listener");

    serve(listener, app)
        .with_graceful_shutdown(check_interupt())
        .await
        .unwrap();
}

#[cfg(debug_assertions)]
pub async fn logger(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    println!("Serving {}", request.uri().to_string());
    next.run(request).await
}

async fn check_interupt() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to launch ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
