use crate::domain::account::AccountId;
use anyhow::Result;
use async_trait::async_trait;
use rusty_money::Money;

pub struct SendMoneyCommand {
    pub source_account_id: AccountId,
    pub target_account_id: AccountId,
    pub money: Money,
}

impl SendMoneyCommand {
    pub fn new(source_account_id: AccountId, target_account_id: AccountId, money: Money) -> Self {
        Self {
            source_account_id,
            target_account_id,
            money,
        }
    }
}

#[async_trait]
pub trait SendMoneyUseCase {
    async fn send_money(&self, command: &SendMoneyCommand) -> Result<bool>;
}
