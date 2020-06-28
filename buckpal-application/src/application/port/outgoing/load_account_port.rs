use crate::domain::account::{Account, AccountId};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait LoadAccountPort {
    async fn load_account(
        &self,
        account_id: &AccountId,
        baseline_date: &DateTime<Utc>,
    ) -> Result<Account>;
}
