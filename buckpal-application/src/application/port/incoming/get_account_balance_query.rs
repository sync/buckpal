use crate::domain::account::AccountId;
use rusty_money::Money;

pub trait GetBalanceQuery {
    fn get_account_balance(account_id: AccountId) -> Money;
}
