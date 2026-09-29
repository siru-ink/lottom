use serde::Serialize;
use sqlx::{Error as SqlxError, PgPool, query_as};

#[derive(Debug, Serialize)]
pub struct PrefillItem {
    id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    euro_price_max: i32,
    euro_price_min: i32,
}

impl PrefillItem {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<Self>, SqlxError> {
        query_as!(PrefillItem, "SELECT * FROM prefill_items")
            .fetch_all(pool)
            .await
    }

    pub async fn read(pool: &PgPool, id: i32) -> Option<Self> {
        query_as!(PrefillItem, "SELECT * FROM prefill_items WHERE id = $1", id)
            .fetch_optional(pool)
            .await
            .ok()?
    }

    pub fn en_name(&self) -> String {
        self.en_name.clone()
    }

    pub fn zh_name(&self) -> String {
        self.zh_name.clone()
    }

    pub fn de_name(&self) -> String {
        self.de_name.clone()
    }
}
