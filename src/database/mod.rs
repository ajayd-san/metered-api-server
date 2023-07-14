mod api_key;
mod ip_book;

use sqlx::{self, PgPool, migrate::MigrateDatabase, Postgres};

pub struct DatabaseMgr {
    pool: PgPool,
}

const DB_URL: &str = "postgres://poweruser:@localhost/databases";

#[derive(Debug, PartialEq)]
pub enum DbResult {
    Ok,
    QueryRes(i32),
}

impl DatabaseMgr {
    pub async fn new() -> Self {
        let pool = Self::setup().await.unwrap();
        DatabaseMgr { pool }
    }

    async fn setup() -> Result<PgPool, sqlx::Error> {
        if !Postgres::database_exists(&DB_URL).await.unwrap_or(false) {
            println!("Database Not found.\nCreating database.");

            match Postgres::create_database(DB_URL).await {
                Ok(_) => println!("create db success"),
                Err(e) => return Err(e),
            }
        }

        let pool = PgPool::connect(DB_URL).await.unwrap();
        sqlx::migrate!().run(&pool).await?;

        Ok(pool)
    }
}
