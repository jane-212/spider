use sqlx::{MySql, Pool, Row};

use crate::error::{IError, IResult};

pub async fn insert_image(pool: &Pool<MySql>, url: &str) -> IResult<()> {
    let sql = "select count(*) from image where url = ?";

    let row = sqlx::query(sql)
        .bind(url)
        .fetch_one(pool)
        .await
        .map_err(|_| IError::Database("select from image failed".into()))?;

    let count: i32 = row
        .try_get(0)
        .map_err(|_| IError::Database("failed to get from row".into()))?;

    if count > 0 {
        return Ok(());
    }

    let sql = "insert into image(url) values(?)";

    let _ = sqlx::query(sql)
        .bind(url)
        .execute(pool)
        .await
        .map_err(|_| IError::Database("insert into image failed".into()))?;

    Ok(())
}
