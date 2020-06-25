use crate::domain::account::{Account, AccountId};
use chrono::{DateTime, Utc};

pub trait LoadAccountPort {
    fn load_account(&self, account_id: &AccountId, baseline_date: &DateTime<Utc>) -> Account;
}
