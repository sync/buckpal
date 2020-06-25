use crate::domain::account::AccountId;

pub trait AccountLock {
    fn lock_account(account_id: AccountId);
    fn release_account(account_id: AccountId);
}
