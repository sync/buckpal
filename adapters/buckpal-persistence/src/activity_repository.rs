use crate::activity_entity::ActivityEntity;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActivityRepositoryError {
    #[error("Something went wrong when summing amount")]
    SumQueryException,
    #[error("Activity already has an id `{0}`, skipping insert")]
    AlreadyHasAnIdException(i32),
}

#[derive(Debug, Clone)]
pub struct ActivityRepository {
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, activity_entity: &ActivityEntity) -> Result<ActivityEntity> {
        match activity_entity.id {
            Some(activity_id) => Err(anyhow!(ActivityRepositoryError::AlreadyHasAnIdException(
                activity_id
            ))),
            None => {
                let entity = sqlx::query_as!(
                    ActivityEntity,
                    r#"
                        INSERT INTO 
                                    activity (timestamp, owner_account_id, source_account_id, target_account_id, amount)
                        VALUES 
                                    ($1, $2, $3, $4, $5)
                        RETURNING 
                                    id, timestamp, owner_account_id, source_account_id, target_account_id, amount 
                    "#,
                    activity_entity.timestamp,
                    activity_entity.owner_account_id,
                    activity_entity.source_account_id,
                    activity_entity.target_account_id,
                    activity_entity.amount
                )
                .fetch_one(&self.pool)
                .await?;

                Ok(entity)
            }
        }
    }

    pub async fn find_by_owner_since(
        &self,
        owner_account_id: i32,
        since: &DateTime<Utc>,
    ) -> Result<Vec<ActivityEntity>> {
        let entitites = sqlx::query_as!(
            ActivityEntity,
            r#"
                SELECT 
                        id,
                        timestamp,
                        owner_account_id,
                        source_account_id,
                        target_account_id,
                        amount
                FROM 
                        activity
                WHERE 
                        owner_account_id = $1
                AND
                        timestamp >= $2
            "#,
            owner_account_id,
            *since
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entitites)
    }

    pub async fn get_deposit_balance_until(
        &self,
        account_id: i32,
        until: &DateTime<Utc>,
    ) -> Result<i64> {
        let sum = sqlx::query!(
            r#"
                SELECT
                        SUM (amount) AS total
                FROM
                        activity
                WHERE
                        target_account_id = $1
                AND 
                        owner_account_id = $1
                AND     
                        timestamp < $2
           "#,
            account_id,
            *until,
        )
        .fetch_one(&self.pool)
        .await?;

        use bigdecimal::*;

        match sum.total.and_then(|total| total.to_i64()) {
            Some(sum) => Ok(sum),
            None => Err(anyhow!(ActivityRepositoryError::SumQueryException)),
        }
    }

    pub async fn get_withdrawal_balance_until(
        &self,
        account_id: i32,
        until: &DateTime<Utc>,
    ) -> Result<i64> {
        let sum = sqlx::query!(
            r#"
                SELECT
                        SUM (amount) AS total
                FROM
                        activity
                WHERE
                        source_account_id = $1
                AND 
                        owner_account_id = $1
                AND     
                        timestamp < $2
           "#,
            account_id,
            *until,
        )
        .fetch_one(&self.pool)
        .await?;

        use bigdecimal::*;

        match sum.total.and_then(|total| total.to_i64()) {
            Some(sum) => Ok(sum),
            None => Err(anyhow!(ActivityRepositoryError::SumQueryException)),
        }
    }
}
