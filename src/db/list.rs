use crate::db::item::Item;
use serde::Serialize;
use sqlx::{Error as SqlxError, PgPool, query, query_as};

#[derive(Debug, Serialize)]
pub struct List {
    id: i32,
    name: String,
}

impl List {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub async fn get_items(&self, pool: &PgPool) -> Result<Vec<Item>, SqlxError> {
        Item::get_items_for_list(pool, self.id).await
    }

    pub async fn get_lists_for_user(pool: &PgPool, user_id: i32) -> Result<Vec<List>, SqlxError> {
        query_as!(
            List,
            "SELECT lists.id, lists.name \
            FROM users \
            JOIN lists_users_mapping ON users.id = lists_users_mapping.user_id
            JOIN lists ON lists_users_mapping.list_id = lists.id
            WHERE users.id = $1",
            user_id,
        )
        .fetch_all(pool)
        .await
    }

    pub fn new(id: i32, name: String) -> Self {
        List { id, name }
    }

    pub async fn create(pool: &PgPool, new_name: &str) -> Option<List> {
        query_as!(
            List,
            "INSERT INTO lists (name) VALUES ($1) RETURNING id, name",
            new_name
        )
        .fetch_optional(pool)
        .await
        .ok()?
    }

    pub async fn read(pool: &PgPool, list_id: i32) -> Option<List> {
        query_as!(List, "SELECT * FROM lists WHERE id = $1", list_id)
            .fetch_optional(pool)
            .await
            .ok()?
    }

    pub async fn update(pool: &PgPool, list: &List) -> Option<List> {
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

    pub async fn delete(pool: &PgPool, list: &List) -> Result<(), SqlxError> {
        match query!("DELETE FROM lists WHERE id = $1", list.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }
}
