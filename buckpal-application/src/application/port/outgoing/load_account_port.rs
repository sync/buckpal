use crate::domain::account::{Account, AccountId};

pub trait LoadAccountPort {
    fn load_account(account_id: AccountId) -> Account;
}
