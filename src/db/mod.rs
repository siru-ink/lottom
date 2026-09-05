use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
    postgres::PgPoolOptions,
};
use std::path::Path;

pub mod item;
pub mod list;
pub mod list_user_map;
mod roles;
pub mod session;
pub mod user;

async fn apply_db_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await
}

pub async fn init_db_connection(postgres_connection_string: &str) -> Option<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(postgres_connection_string)
        .await
        .expect("Database should be reachable.");

    if let Err(e) = apply_db_migrations(&pool).await {
        println!("Error appliying database migrations: {}", e);
    };

    Some(pool)
}
