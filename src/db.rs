use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPool;
use sqlx::query_as;
use std::path::Path;

pub async fn apply_db_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await
}

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
    password: String,
    default_list_id: i32,
}

async fn read_user(pool: &PgPool, user_id: i32) -> Option<User> {
    query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(pool)
        .await
        .ok()?
}
