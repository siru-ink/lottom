use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
};
use std::path::Path;

pub mod item;
pub mod list;
pub mod session;
pub mod user;

pub async fn apply_db_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await
}
