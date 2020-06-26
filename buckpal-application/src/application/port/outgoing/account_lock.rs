use crate::domain::account::AccountId;

#[cfg_attr(test, mockall::automock)]
pub trait AccountLock {
    fn lock_account(&self, account_id: &AccountId);
    fn release_account(&self, account_id: &AccountId);
}
