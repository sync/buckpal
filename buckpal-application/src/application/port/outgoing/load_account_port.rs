use crate::domain::account::{Account, AccountId};
use chrono::{DateTime, Utc};

#[cfg_attr(test, mockall::automock)]
pub trait LoadAccountPort {
    fn load_account(&self, account_id: &AccountId, baseline_date: &DateTime<Utc>) -> Account;
}
