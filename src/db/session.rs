use crate::{cookies::CookieValue, db::user::User};
use chrono::{DateTime, Utc};
use sqlx::{Error as SqlxError, PgPool, query, query_as};

#[derive(Debug)]
pub struct Session {
    id: i32,
    user_id: i32,
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

    pub async fn update(pool: &PgPool, session: &Session) -> Option<Session> {
        query_as!(
            Session,
            "UPDATE sessions \
             SET user_id = $1, start = $2 \
             WHERE id = $3 \
             RETURNING id, user_id, start",
            session.user_id,
            session.start,
            session.id
        )
        .fetch_optional(pool)
        .await
        .ok()?
    }

    pub async fn delete(pool: &PgPool, session: &Session) -> Result<(), SqlxError> {
        match query!("DELETE FROM sessions WHERE id = $1", session.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
