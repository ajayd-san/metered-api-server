mod api_key;
mod ip_book;

use sqlx::{self, SqlitePool};

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
        //TODO: set max pool connnections to 8.
        let pool = SqlitePool::connect(DB_URL).await.unwrap();
        DatabaseMgr { pool }
    }
}

