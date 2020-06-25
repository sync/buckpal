use crate::domain::account::AccountId;
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

pub trait SendMoneyUseCase {
    fn send_money(command: SendMoneyCommand) -> bool;
}
