use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

use crate::error::{AppError, AppResult};

pub async fn init_db(database_path: &Path) -> AppResult<SqlitePool> {
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_url = format!(
        "sqlite:///{}",
        database_path.to_string_lossy().replace('\\', "/")
    );

    let options = SqliteConnectOptions::from_str(&db_url)?
        .foreign_keys(true)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(|e| AppError::Database(sqlx::Error::Configuration(e.to_string().into())))?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    // SQL 文件不是运行时从磁盘读取，而是在编译时就已经被"打包"进可执行文件中
    let migration_001 = include_str!("../../migrations/001_workspace.sql");

    sqlx::raw_sql(migration_001)
        .execute(pool)
        .await?;

    tracing::info!("Database migrations completed");
    Ok(())
}
