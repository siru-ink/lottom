use crate::db::item::Item;
use sqlx::{Error as SqlxError, PgPool, query, query_as};

#[derive(Debug)]
pub struct List {
    id: i32,
    name: String,
}

impl List {
    pub fn get_id(&self) -> i32 {
        self.id
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

    pub async fn get_items(&self, pool: &PgPool) -> Result<Vec<Item>, SqlxError> {
        Item::get_items_for_list(pool, self.id).await
    }
}
