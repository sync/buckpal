use crate::domain::account::Account;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait UpdateAccountStatePort {
    async fn update_activities(&self, account: &Account) -> Result<()>;
}
