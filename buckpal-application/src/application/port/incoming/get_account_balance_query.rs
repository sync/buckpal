use crate::domain::account::AccountId;
use rusty_money::Money;

pub trait GetAccountBalanceQuery {
    fn get_account_balance(&self, account_id: &AccountId) -> Money;
}
