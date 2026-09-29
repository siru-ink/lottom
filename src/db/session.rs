use crate::{cookies::CookieValue, db::user::User};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query_as};

#[derive(Debug)]
pub struct Session {
    id: i32,
    user_id: i32,
    #[allow(dead_code)]
    start: DateTime<Utc>,
}

impl Session {
    pub fn as_cookie_value(&self) -> CookieValue {
        CookieValue::SessionID(self.id)
    }

    pub async fn get_user(&self, pool: &PgPool) -> Option<User> {
        User::read(pool, self.user_id).await
    }

    pub async fn create(pool: &PgPool, user: &User) -> Option<Session> {
        query_as!(
            Session,
            "INSERT INTO sessions (user_id, start) VALUES ($1, now()) RETURNING id, user_id, start",
            user.get_id()
        )
        .fetch_optional(pool)
        .await
        .ok()?
    }

    pub async fn read(pool: &PgPool, session_id: i32) -> Option<Session> {
        query_as!(Session, "SELECT * FROM sessions WHERE id = $1", session_id)
            .fetch_optional(pool)
            .await
            .ok()?
    }
}
