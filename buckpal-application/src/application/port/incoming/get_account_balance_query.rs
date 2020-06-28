use crate::domain::account::AccountId;
use anyhow::Result;
use async_trait::async_trait;
use rusty_money::Money;

#[async_trait]
pub trait GetAccountBalanceQuery {
    async fn get_account_balance(&self, account_id: &AccountId) -> Result<Money>;
}
