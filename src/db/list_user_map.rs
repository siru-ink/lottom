use serde::Serialize;
use sqlx::{PgPool, query_as};

#[derive(Debug, Serialize)]
pub struct ListUserMapping {
    user_id: i32,
    list_id: i32,
    role_id: i32,
}

impl ListUserMapping {
    pub async fn create(
        pool: &PgPool,
        user_id: i32,
        list_id: i32,
        role_id: i32,
    ) -> Option<ListUserMapping> {
        query_as!(
            ListUserMapping,
            "INSERT INTO lists_users_mapping (user_id, list_id, role_id) \
             VALUES ($1, $2, $3) \
             RETURNING user_id, list_id, role_id",
            user_id,
            list_id,
            role_id
        )
        .fetch_optional(pool)
        .await
        .ok()?
    }
}
