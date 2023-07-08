use sqlx::{self, sqlite::SqliteQueryResult, Row, SqlitePool, query};

use crate::KeyRegistarationData;

use crate::database::{DatabaseMgr, DbResult};

impl DatabaseMgr {
    pub async fn add_api_key(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            INSERT INTO keys (api_key, queries_left) VALUES ($1, $2)
            ",
            key.key,
            key.quota_per_min
        )
        .execute(&self.pool)
        .await?;

        Ok(DbResult::Ok)
    }

    pub async fn update_quota_api_key(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            UPDATE keys SET queries_left = queries_left - 1 WHERE api_key = $1
            ",
            key.key
        )
        .execute(&self.pool)
        .await?;

        // Ok(res.queries_left.unwrap().try_into().unwrap())
        Ok(DbResult::Ok)
    }

    pub async fn check_quota_api_key(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        let res = sqlx::query!(
            "
            SELECT queries_left FROM keys WHERE api_key = $1
            ",
            key.key
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DbResult::QueryRes(res.queries_left.unwrap() as u32))
        // Err(sqlx::Error::RowNotFound)
    }

    pub async fn reset_quota_api_key(&self) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            UPDATE keys SET queries_left = 10;
            "
            ).execute(&self.pool).await?;

        Ok(DbResult::Ok)
    }
}

#[cfg(test)]
mod tests {
    use crate::Db;

    use super::*;

    #[tokio::test]
    async fn test_update_query() {
        let dbm = DatabaseMgr::new().await;
        let key = KeyRegistarationData {
            key: "idk".to_string(),
            quota_per_min: 1,
            db_name: Db::API_KEY
        };
        let before = dbm.check_quota_api_key(&key).await.unwrap();
        dbm.update_quota_api_key(&key).await.unwrap();
        let after = dbm.check_quota_api_key(&key).await.unwrap();

        let mut res = Vec::new();

        for val in [before, after] {
            match val {
                DbResult::QueryRes(i) => res.push(i),
                _ => unreachable!()
            }
        }
        assert_eq!(*res.get(0).unwrap() - 1, *res.get(1).unwrap());
    }

    #[tokio::test]
    async fn test_add_key_fails() {
        let dbm = DatabaseMgr::new().await;
        let key = KeyRegistarationData {
            key: "123".to_string(),
            quota_per_min: 1,
            db_name: Db::IP_BOOK
        };
        assert!(dbm.add_api_key(&key).await.is_err());
    }
}

