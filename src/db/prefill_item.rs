use serde::Serialize;
use sqlx::{Error as SqlxError, PgPool, query_as};

#[derive(Debug, Serialize)]
pub struct PrefillItem {
    id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    euro_price: i32,
}

impl PrefillItem {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<Self>, SqlxError> {
        query_as!(PrefillItem, "SELECT * FROM prefill_items")
            .fetch_all(pool)
            .await
    }
}
