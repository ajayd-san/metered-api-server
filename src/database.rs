use sqlx::{self, sqlite::SqliteQueryResult, SqlitePool};

use crate::KeyRegistarationData;

struct DatabaseMgr {
    pool: SqlitePool,
}

const DB_URL: &str = "sqlite://api_key_data.db";

impl DatabaseMgr {
    async fn new() -> Self {
        //TODO: set max pool connnections to 8.
        let pool = SqlitePool::connect(DB_URL).await.unwrap();
        DatabaseMgr { pool }
    }

    async fn add_key(&self, key: &KeyRegistarationData) -> sqlx::Result<()> {
        sqlx::query!(
            "
            INSERT INTO keys (api_key, queries_left) VALUES ($1, $2)
            ",
            key.api_key,
            key.quota_per_min
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_quota(&self, key: &KeyRegistarationData) -> sqlx::Result<()> {
        sqlx::query!(
            "
            UPDATE keys SET queries_left = queries_left - 1 WHERE api_key = $1
            ",
            key.api_key
        )
        .execute(&self.pool)
        .await?;

        // Ok(res.queries_left.unwrap().try_into().unwrap())
        Ok(())
    }

    async fn check_quota(&self, key: &KeyRegistarationData) -> sqlx::Result<u32> {
        let res = sqlx::query!(
            "
            SELECT queries_left FROM keys WHERE api_key = $1
            ",
            key.api_key
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();

        Ok(res.queries_left.unwrap() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_query() {
        let dbm = DatabaseMgr::new().await;
        let key = KeyRegistarationData {
            api_key: "idk".to_string(),
            quota_per_min: 1,
        };
        let before = dbm.check_quota(&key).await.unwrap();
        dbm.update_quota(&key).await.unwrap();
        let after = dbm.check_quota(&key).await.unwrap();
        assert_eq!(after, before - 1);
    }

    #[tokio::test]
    async fn test_add_key_fails() {
        let dbm = DatabaseMgr::new().await;
        let key = KeyRegistarationData {
            api_key: "123".to_string(),
            quota_per_min: 1,
        };
        assert!(dbm.add_key(&key).await.is_err());
    }
}
