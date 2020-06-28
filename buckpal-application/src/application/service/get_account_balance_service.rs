use crate::application::port::incoming::get_account_balance_query::GetAccountBalanceQuery;
use crate::application::port::outgoing::load_account_port::LoadAccountPort;
use crate::domain::account::AccountId;
use anyhow::Result;
use async_trait::async_trait;
use rusty_money::Money;

pub struct GetAccountBalanceService<'a> {
    load_account_port: Box<dyn LoadAccountPort + Sync + Send + 'a>,
}

#[async_trait]
impl<'a> GetAccountBalanceQuery for GetAccountBalanceService<'a> {
    async fn get_account_balance(&self, account_id: &AccountId) -> Result<Money> {
        use chrono::Utc;

        let account = self
            .load_account_port
            .load_account(account_id, &Utc::now())
            .await?;

        Ok(account.calculate_balance())
    }
}
