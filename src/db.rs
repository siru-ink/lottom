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
    let new_list = match create_list(pool, "default").await {
        Some(val) => val,
        None => return None,
    };

    let new_user_id = query!(
        "INSERT INTO users (name, password, default_list_id) VALUES ($1, $2, $3) RETURNING id",
        new_name,
        new_password,
        new_list.id
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

async fn read_list(pool: &PgPool, list_id: i32) -> Option<List> {
    query_as!(List, "SELECT * FROM lists WHERE id = $1", list_id)
        .fetch_optional(pool)
        .await
        .ok()?
}

async fn update_list(pool: &PgPool, list: &List) -> Option<List> {
    query_as!(
        List,
        "UPDATE lists SET name = $1 WHERE id = $2 RETURNING id,name",
        list.name,
        list.id
    )
    .fetch_optional(pool)
    .await
    .ok()?
}

async fn delete_list(pool: &PgPool, list: &List) -> Result<(), Error> {
    match query!("DELETE FROM lists WHERE id = $1", list.id)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(e),
    }
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

async fn create_item(
    pool: &PgPool,
    new_list_id: i32,
    new_en_name: &String,
    new_zh_name: &String,
    new_de_name: &String,
    optional_new_img_path: &Option<String>,
    new_estimated_euro_price: i32,
) -> Option<Item> {
    match optional_new_img_path {
        None => {
            query_as!(
                Item,
                "INSERT INTO items (list_id, en_name, zh_name, de_name, estimated_euro_price) \
                 VALUES ($1, $2, $3, $4, $5) \
                 RETURNING id, list_id, en_name, zh_name, de_name, img_path, estimated_euro_price",
                new_list_id,
                new_en_name,
                new_zh_name,
                new_de_name,
                new_estimated_euro_price
            ).fetch_optional(pool).await.ok()?
        }
        Some(new_img_path) => {
            query_as!(
                Item,
                "INSERT INTO items (list_id, en_name, zh_name, de_name, img_path, estimated_euro_price) \
                 VALUES ($1, $2, $3, $4, $5, $6) \
                 RETURNING id, list_id, en_name, zh_name, de_name, img_path, estimated_euro_price",
                new_list_id,
                new_en_name,
                new_zh_name,
                new_de_name,
                new_img_path,
                new_estimated_euro_price
            ).fetch_optional(pool).await.ok()?
        }
    }
}

async fn read_item(pool: &PgPool, item_id: i32) -> Option<Item> {
    query_as!(Item, "SELECT * FROM items WHERE id = $1", item_id)
        .fetch_optional(pool)
        .await
        .ok()?
}

async fn update_item(pool: &PgPool, item: &Item) -> Option<Item> {
    query_as!(
        Item,
        "UPDATE items \
         SET list_id = $1, en_name = $2, zh_name = $3, de_name = $4, img_path = $5, estimated_euro_price = $6 \
         WHERE id = $7 \
         RETURNING id, list_id, en_name, zh_name, de_name, img_path, estimated_euro_price",
        item.list_id,
        item.en_name,
        item.zh_name,
        item.de_name,
        item.img_path,
        item.estimated_euro_price,
        item.id
    ).fetch_optional(pool).await.ok()?
}

async fn delete_item(pool: &PgPool, item: &Item) -> Result<(), Error> {
    match query!("DELETE FROM items WHERE id = $1", item.id)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(e),
    }
}
