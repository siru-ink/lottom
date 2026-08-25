use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPool;
use sqlx::{Error, query, query_as};
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

async fn create_user(pool: &PgPool, new_name: &str, new_password: &str) -> Option<i32> {
    let new_list_id = query!("INSERT INTO lists (name) VALUES ('default') RETURNING id")
        .fetch_one(pool)
        .await
        .ok()?
        .id;

    let new_user_id = query!(
        "INSERT INTO users (name, password, default_list_id) VALUES ($1, $2, $3) RETURNING id",
        new_name,
        new_password,
        new_list_id
    )
    .fetch_one(pool)
    .await
    .ok()?
    .id;

    Some(new_user_id)
}

async fn read_user(pool: &PgPool, user_id: i32) -> Option<User> {
    query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(pool)
        .await
        .ok()?
}

async fn update_user(pool: &PgPool, user: &User) -> Option<User> {
    query_as!(
        User,
        "UPDATE users SET name = $1, password = $2, default_list_id = $3 WHERE id = $4 RETURNING id, name, password, default_list_id",
        user.name,
        user.password,
        user.default_list_id,
        user.id
    )
    .fetch_optional(pool)
    .await
    .ok()?
}

async fn delete_user(pool: &PgPool, user: &User) -> Result<(), Error> {
    match query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
struct List {
    id: i32,
    name: String,
}

async fn create_list(pool: &PgPool, new_name: &str) -> Option<List> {
    query_as!(
        List,
        "INSERT INTO lists (name) VALUES ($1) RETURNING id, name",
        new_name
    )
    .fetch_optional(pool)
    .await
    .ok()?
}

#[derive(Debug)]
struct Item {
    id: i32,
    list_id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    img_path: Option<String>,
    estimated_euro_price: i32,
}
