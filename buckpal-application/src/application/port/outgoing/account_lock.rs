use crate::domain::account::AccountId;

pub trait AccountLock {
    fn lock_account(&self, account_id: &AccountId);
    fn release_account(&self, account_id: &AccountId);
}
