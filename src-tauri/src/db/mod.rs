pub mod init;

pub use init::init_db;

use sqlx::SqlitePool;

pub type DatabasePool = SqlitePool;
