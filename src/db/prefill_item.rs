use serde::Serialize;
use sqlx::{Error as SqlxError, PgPool, Postgres, QueryBuilder, query_as};

#[derive(Debug, Serialize)]
pub struct PrefillItem {
    id: i32,
    en_name: String,
    zh_name: String,
    de_name: String,
    euro_price_max: i32,
    euro_price_min: i32,
}

pub struct PartialPrefillItem {
    pub en_name: String,
    pub zh_name: String,
    pub de_name: String,
    pub cents_price_max: i32,
    pub cents_price_min: i32,
}

impl PrefillItem {
    pub async fn update_items(
        pool: &PgPool,
        items: Vec<PartialPrefillItem>,
    ) -> Result<(), SqlxError> {
        if items.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO prefill_items (en_name, zh_name, de_name, euro_price_max, euro_price_min) ",
        );

        builder.push_values(items, |mut qb, item| {
            qb.push_bind(item.en_name)
                .push_bind(item.zh_name)
                .push_bind(item.de_name)
                .push_bind(item.cents_price_max)
                .push_bind(item.cents_price_min);
        });

        builder.push(
            " ON CONFLICT (en_name) DO UPDATE SET \
             zh_name = EXCLUDED.zh_name, \
             de_name = EXCLUDED.de_name, \
             euro_price_max = EXCLUDED.euro_price_max, \
             euro_price_min = EXCLUDED.euro_price_min",
        );

        match builder.build().execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

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

    pub fn average_price(&self) -> i32 {
        (self.euro_price_max + self.euro_price_min) / 2
    }
}
