use crate::account_entity::AccountEntity;
use anyhow::Result;
use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, account_id: i32) -> Result<AccountEntity> {
        let entity = sqlx::query_as!(
            AccountEntity,
            r#"
                SELECT
                        id 
                FROM 
                        account
                WHERE 
                        id = $1
            "#,
            account_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }
}
