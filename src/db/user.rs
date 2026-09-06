use crate::db::{list::List, list_user_map::ListUserMapping, roles::Roles};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sqlx::{Error as SqlxError, PgPool, query, query_as};

#[derive(Debug)]
pub struct User {
    id: i32,
    name: String,
    password: String,
    default_list_id: i32,
}

pub enum CheckPasswordResult {
    PasswordCorrect,
    PasswordWrong,
}

impl User {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn check_password(&self, comparison_password: &String) -> CheckPasswordResult {
        let parsed_hash = PasswordHash::new(&self.password).unwrap();

        match Argon2::default().verify_password(comparison_password.as_bytes(), &parsed_hash) {
            Ok(_) => CheckPasswordResult::PasswordCorrect,
            Err(_) => CheckPasswordResult::PasswordWrong,
        }
    }

    pub fn default_list(&self) -> i32 {
        self.default_list_id
    }

    pub async fn lists(&self, pool: &PgPool) -> Result<Vec<List>, SqlxError> {
        List::get_lists_for_user(pool, self.id).await
    }

    pub async fn create(pool: &PgPool, new_name: &str, new_password: &str) -> Option<i32> {
        let new_list = match List::create(pool, "default").await {
            Some(val) => val,
            None => return None,
        };

        let argon2 = Argon2::default();
        let new_password_bytes = new_password.as_bytes();
        let password_hash = argon2
            .hash_password(new_password_bytes)
            .unwrap()
            .to_string();

        println!("{}", password_hash);

        let new_user_id = query!(
            "INSERT INTO users (name, password, default_list_id) VALUES ($1, $2, $3) RETURNING id",
            new_name,
            password_hash,
            new_list.get_id()
        )
        .fetch_one(pool)
        .await
        .ok()?
        .id;

        // Also create the lists <-> users mapping
        ListUserMapping::create(pool, new_user_id, new_list.get_id(), Roles::Owner.id()).await;

        // TODO add rollback logic in case either of the inserts fails to leave db in better state

        Some(new_user_id)
    }

    pub async fn read(pool: &PgPool, user_id: i32) -> Option<User> {
        query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(pool)
            .await
            .ok()?
    }

    pub async fn update(pool: &PgPool, user: &User) -> Option<User> {
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

    pub async fn delete(pool: &PgPool, user: &User) -> Result<(), SqlxError> {
        match query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
