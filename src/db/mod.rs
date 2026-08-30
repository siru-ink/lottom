use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
};
use std::path::Path;

mod item;
mod list;
mod session;
mod user;

pub async fn apply_db_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await
}
