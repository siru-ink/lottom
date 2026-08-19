use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPool;
use std::path::Path;

pub async fn apply_db_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await
}
