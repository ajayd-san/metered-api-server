use crate::KeyRegistarationData;

use super::{DatabaseMgr, DbResult};

impl DatabaseMgr {
    pub async fn add_ip(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            INSERT INTO ip_book (ip, queries_left) VALUES ($1, $2)
            ",
            key.key,
            key.quota
        )
        .execute(&self.pool)
        .await?;

        Ok(DbResult::Ok)
    }

    pub async fn update_quota_ip(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            UPDATE ip_book SET queries_left = queries_left - 1 WHERE ip = $1
            ",
            key.key
        )
        .execute(&self.pool)
        .await?;

        // Ok(res.queries_left.unwrap().try_into().unwrap())
        Ok(DbResult::Ok)
    }

    pub async fn check_quota_ip(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        let res = sqlx::query!(
            "
            SELECT queries_left FROM ip_book WHERE ip = $1
            ",
            key.key
        )
        .fetch_one(&self.pool)
        .await;

        if res.is_err() {
            self.add_ip(&key).await.unwrap();
            return Ok(DbResult::QueryRes(10));
        }

        Ok(DbResult::QueryRes(res.unwrap().queries_left.unwrap()))
    }

    pub async fn reset_quota_ip(&self) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            UPDATE ip_book SET queries_left = 10;
            "
            ).execute(&self.pool).await?;

        Ok(DbResult::Ok)
    }
}
