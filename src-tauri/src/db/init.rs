use sqlx::SqlitePool;
use std::path::Path;

use crate::error::{AppError, AppResult};

pub async fn init_db(database_path: &Path) -> AppResult<SqlitePool> {
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_url = format!(
        "sqlite:{}?foreign_keys=on",
        database_path.to_string_lossy()
    );

    let pool = SqlitePool::connect(&db_url).await.map_err(|e| {
        AppError::Database(sqlx::Error::Configuration(e.to_string().into()))
    })?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    let migration_sql = include_str!("../../migrations/001_initial.sql");

    sqlx::raw_sql(migration_sql)
        .execute(pool)
        .await?;

    tracing::info!("Database migrations completed");
    Ok(())
}
