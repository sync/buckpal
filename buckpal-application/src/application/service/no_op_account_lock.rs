use crate::application::port::outgoing::account_lock::AccountLock;
use crate::domain::account::AccountId;

pub struct NoOpAccountLock {}

impl AccountLock for NoOpAccountLock {
    fn lock_account(&self, _account_id: &AccountId) {
        // do nothing
    }
    fn release_account(&self, _account_id: &AccountId) {
        // do nothing
    }
}
