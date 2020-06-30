use crate::application::port::outgoing::account_lock::AccountLock;
use crate::domain::account::AccountId;

#[derive(Debug, Clone)]
pub struct NoOpAccountLock {}

impl NoOpAccountLock {
    pub fn new() -> Self {
        Self {}
    }
}

impl AccountLock for NoOpAccountLock {
    fn lock_account(&self, _account_id: &AccountId) {
        // do nothing
    }
    fn release_account(&self, _account_id: &AccountId) {
        // do nothing
    }
}
