mod api_key;
mod ip_book;

use std::env;

use sqlx::{self, PgPool, migrate::MigrateDatabase, Postgres};

pub struct DatabaseMgr {
    pool: PgPool,
}


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
        let username = env::var("PG_USER").unwrap();
        let host = env::var("HOST").unwrap();
        let password = std::fs::read_to_string("/run/secrets/db_password").unwrap();


        let db_url= format!("postgres://{}:{}@{}/databases", username, password, host);
        if !Postgres::database_exists(&db_url).await.unwrap_or(false) {
            println!("Database Not found.\nCreating database.");

            match Postgres::create_database(&db_url).await {
                Ok(_) => println!("create db success"),
                Err(e) => return Err(e),
            }
        }

        let pool = PgPool::connect(&db_url).await.unwrap();
        sqlx::migrate!().run(&pool).await?;

        Ok(pool)
    }
}
