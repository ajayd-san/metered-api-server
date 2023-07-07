use metered_api_server::KeyRegistarationData;

use super::{DatabaseMgr, DbResult};

impl DatabaseMgr {
    pub async fn add_ip(&self, key: &KeyRegistarationData) -> sqlx::Result<DbResult> {
        sqlx::query!(
            "
            INSERT INTO ip_book (ip, queries_left) VALUES ($1, $2)
            ",
            key.key,
            key.quota_per_min
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
        .await?;

        Ok(DbResult::QueryRes(res.queries_left.unwrap() as u32))
        // Err(sqlx::Error::RowNotFound)
    }
}
