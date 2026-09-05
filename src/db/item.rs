use serde::Serialize;
use sqlx::{Error as SqlxError, PgPool, query, query_as};

#[derive(Debug, Serialize)]
pub struct Item {
    id: i32,
    list_id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    img_path: Option<String>,
    estimated_euro_price: i32,
}

impl Item {
    pub fn new(
        id: i32,
        list_id: i32,
        en_name: String,
        zh_name: String,
        de_name: String,
        img_path: Option<String>,
        estimated_euro_price: i32,
    ) -> Self {
        Item {
            id,
            list_id,
            en_name,
            zh_name,
            de_name,
            img_path,
            estimated_euro_price,
        }
    }

    pub fn id(self) -> i32 {
        self.id
    }

    pub async fn create(
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

    pub async fn read(pool: &PgPool, item_id: i32) -> Option<Item> {
        query_as!(Item, "SELECT * FROM items WHERE id = $1", item_id)
            .fetch_optional(pool)
            .await
            .ok()?
    }

    pub async fn update(pool: &PgPool, item: &Item) -> Option<Item> {
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

    pub async fn delete(pool: &PgPool, item: &Item) -> Result<(), SqlxError> {
        match query!("DELETE FROM items WHERE id = $1", item.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    pub async fn get_items_for_list(pool: &PgPool, list_id: i32) -> Result<Vec<Item>, SqlxError> {
        query_as!(Item, "SELECT * FROM items WHERE list_id = $1", list_id)
            .fetch_all(pool)
            .await
    }
}
