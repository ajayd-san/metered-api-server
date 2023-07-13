mod api_key;
mod ip_book;

use std::{fs, path::Path};

use sqlx::{self, SqlitePool, Sqlite, migrate::MigrateDatabase};

pub struct DatabaseMgr {
    pool: SqlitePool,
}

const DB_URL: &str = "sqlite://api_key_data.db";

#[derive(Debug, PartialEq)]
pub enum DbResult {
    Ok,
    QueryRes(u32),
}

impl DatabaseMgr {
    pub async fn new() -> Self {
        let pool = Self::setup().await.unwrap();
        DatabaseMgr { pool }
    }

    async fn setup() -> Result<SqlitePool, sqlx::Error> {
        if !Sqlite::database_exists(&DB_URL).await.unwrap_or(false) {
            println!("Database Not found.\nCreating database.");

            match Sqlite::create_database(DB_URL).await {
                Ok(_) => println!("create db success"),
                Err(e) => return Err(e),
            }
        }

        let pool = SqlitePool::connect(DB_URL).await.unwrap();
        sqlx::migrate!().run(&pool).await?;

        Ok(pool)
    }
}
