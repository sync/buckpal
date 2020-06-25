use crate::domain::account::AccountId;
use chrono::{DateTime, Utc};
use rusty_money::Money;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ActivityId(i32);

/// A money transfer activity between Accounts
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Activity {
    pub id: Option<ActivityId>,
    /// The account that owns this activity.
    pub owner_account_id: AccountId,
    /// The debited account.
    pub source_account_id: AccountId,
    /// The credited account.
    pub target_account_id: AccountId,
    /// The timestamp of the activity.
    pub timestamp: DateTime<Utc>,
    /// The money that was transferred between the accounts.
    pub money: Money,
}

impl Activity {
    pub fn new(
        owner_account_id: AccountId,
        source_account_id: AccountId,
        target_account_id: AccountId,
        timestamp: DateTime<Utc>,
        money: Money,
    ) -> Self {
        Self {
            id: None,
            owner_account_id,
            source_account_id,
            target_account_id,
            timestamp,
            money,
        }
    }
}

#[cfg(test)]
pub mod activity_test_data {
    use super::{AccountId, Activity};
    use chrono::{DateTime, Utc};
    use rusty_money::{money, Money};

    pub struct ActivityBuilder {
        activity: Activity,
    }

    impl ActivityBuilder {
        pub fn default_activity() -> Self {
            let activity = Activity::new(
                AccountId(42),
                AccountId(42),
                AccountId(41),
                Utc::now(),
                money!(999, "AUD"),
            );

            Self { activity }
        }

        pub fn with_timestamp(&mut self, timestamp: DateTime<Utc>) -> &mut Self {
            let mut activity = self.activity.clone();
            activity.timestamp = timestamp;

            let mut new = self;
            new.activity = activity;
            new
        }

        pub fn with_source_account(&mut self, source_account_id: AccountId) -> &mut Self {
            let mut activity = self.activity.clone();
            activity.source_account_id = source_account_id;

            let mut new = self;
            new.activity = activity;
            new
        }

        pub fn with_target_account(&mut self, target_account_id: AccountId) -> &mut Self {
            let mut activity = self.activity.clone();
            activity.target_account_id = target_account_id;

            let mut new = self;
            new.activity = activity;
            new
        }

        pub fn with_money(&mut self, money: Money) -> &mut Self {
            let mut activity = self.activity.clone();
            activity.money = money;

            let mut new = self;
            new.activity = activity;
            new
        }

        pub fn build(&self) -> Activity {
            self.activity.clone()
        }
    }
}
