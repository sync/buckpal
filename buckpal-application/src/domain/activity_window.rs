use crate::domain::account::AccountId;
use crate::domain::activity::Activity;
use chrono::{DateTime, Utc};
use rusty_money::{money, Money};

/// A window of account activities.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ActivityWindow {
    /// The list of account activities within this window.
    pub activities: Vec<Activity>,
}

impl ActivityWindow {
    pub fn new(activities: Vec<Activity>) -> Self {
        Self { activities }
    }

    /// The timestamp of the first activity within this window.
    pub fn get_start_timestamp(&self) -> Option<DateTime<Utc>> {
        self.activities
            .clone()
            .into_iter()
            .map(|activity| activity.timestamp)
            .min()
    }

    /// The timestamp of the last activity within this window.
    pub fn get_end_timestamp(&self) -> Option<DateTime<Utc>> {
        self.activities
            .clone()
            .into_iter()
            .map(|activity| activity.timestamp)
            .max()
    }

    /// Calculates the balance by summing up the values of all activities within this window.
    pub fn calculate_balance(&self, account_id: &AccountId) -> Money {
        let deposit_balance = self
            .activities
            .clone()
            .into_iter()
            .filter_map(|activity| {
                if activity.target_account_id == *account_id {
                    Some(activity.money)
                } else {
                    None
                }
            })
            .fold(money!(0, "AUD"), |acc, x| acc + x);

        let withdrawal_balance = self
            .activities
            .clone()
            .into_iter()
            .filter_map(|activity| {
                if activity.source_account_id == *account_id {
                    Some(activity.money)
                } else {
                    None
                }
            })
            .fold(money!(0, "AUD"), |acc, x| acc + x);

        deposit_balance - withdrawal_balance
    }

    pub fn add_activity(&mut self, activity: &Activity) {
        self.activities.push(activity.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountId, ActivityWindow};
    use crate::domain::activity::activity_test_data::ActivityBuilder;
    use chrono::{DateTime, NaiveDate, Utc};
    use rusty_money::{money, Money};

    fn start_date() -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 3).and_hms(0, 0, 0), Utc)
    }

    fn in_between_date() -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 4).and_hms(0, 0, 0), Utc)
    }

    fn end_date() -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 5).and_hms(0, 0, 0), Utc)
    }

    #[test]
    fn calculates_start_timestamp() {
        let activity_now = ActivityBuilder::default_activity()
            .with_timestamp(&start_date())
            .build();

        let activity_between = ActivityBuilder::default_activity()
            .with_timestamp(&in_between_date())
            .build();

        let activity_tomorrow = ActivityBuilder::default_activity()
            .with_timestamp(&end_date())
            .build();

        let window = ActivityWindow::new(vec![activity_now, activity_between, activity_tomorrow]);

        assert_eq!(window.get_start_timestamp().unwrap(), start_date());
    }

    #[test]
    fn calculates_end_timestamp() {
        let activity_now = ActivityBuilder::default_activity()
            .with_timestamp(&start_date())
            .build();

        let activity_between = ActivityBuilder::default_activity()
            .with_timestamp(&in_between_date())
            .build();

        let activity_tomorrow = ActivityBuilder::default_activity()
            .with_timestamp(&end_date())
            .build();

        let window = ActivityWindow::new(vec![activity_now, activity_between, activity_tomorrow]);

        assert_eq!(window.get_end_timestamp().unwrap(), end_date());
    }

    #[test]
    fn calculates_balance() {
        let account1 = AccountId(1);
        let account2 = AccountId(2);

        let window = ActivityWindow::new(vec![
            ActivityBuilder::default_activity()
                .with_source_account(&account1)
                .with_target_account(&account2)
                .with_money(&money!(999, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_source_account(&account1)
                .with_target_account(&account2)
                .with_money(&money!(1, "AUD"))
                .build(),
            ActivityBuilder::default_activity()
                .with_source_account(&account2)
                .with_target_account(&account1)
                .with_money(&money!(500, "AUD"))
                .build(),
        ]);

        debug_assert_eq!(window.calculate_balance(&account1), money!(-500, "AUD"));
        debug_assert_eq!(window.calculate_balance(&account2), money!(500, "AUD"));
    }
}
