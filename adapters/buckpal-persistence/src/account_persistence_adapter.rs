use crate::account_mapper::AccountMapper;
use crate::account_repository::AccountRepository;
use crate::activity_repository::ActivityRepository;
use anyhow::Result;
use async_trait::async_trait;
use buckpal_application::application::port::outgoing::{
    load_account_port::LoadAccountPort, update_account_state_port::UpdateAccountStatePort,
};
use buckpal_application::domain::account::{Account, AccountId};
use buckpal_application::domain::activity::Activity;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct AccountPersistenceAdapter {
    account_repository: AccountRepository,
    activity_repository: ActivityRepository,
    account_mapper: AccountMapper,
}

impl AccountPersistenceAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self {
            account_repository: AccountRepository::new(pool.clone()),
            activity_repository: ActivityRepository::new(pool),
            account_mapper: AccountMapper::default(),
        }
    }
}

#[async_trait]
impl LoadAccountPort for AccountPersistenceAdapter {
    async fn load_account(
        &self,
        account_id: &AccountId,
        baseline_date: &DateTime<Utc>,
    ) -> Result<Account> {
        let account_entity = self.account_repository.find_by_id(account_id.0).await?;

        let activities = self
            .activity_repository
            .find_by_owner_since(account_id.0, baseline_date)
            .await?;

        let withdrawal_balance = self
            .activity_repository
            .get_withdrawal_balance_until(account_id.0, baseline_date)
            .await
            .unwrap_or(0);

        let deposit_balance = self
            .activity_repository
            .get_deposit_balance_until(account_id.0, baseline_date)
            .await
            .unwrap_or(0);

        let account = self.account_mapper.map_to_domain_entity(
            account_entity,
            activities,
            withdrawal_balance,
            deposit_balance,
        );

        Ok(account)
    }
}

#[async_trait]
impl UpdateAccountStatePort for AccountPersistenceAdapter {
    async fn update_activities(&self, account: &Account) -> Result<Vec<Activity>> {
        let mut activities: Vec<Activity> = vec![];
        for activity in account.clone().activity_window.activities {
            if activity.id.is_none() {
                let activity_entity = self
                    .activity_repository
                    .save(&self.account_mapper.map_to_entity(activity))
                    .await?;
                activities.push(self.account_mapper.map_to_activity(&activity_entity));
            }
        }

        Ok(activities)
    }
}

#[cfg(test)]
mod tests {
    use super::AccountPersistenceAdapter;
    use crate::activity_entity::ActivityEntity;
    use crate::activity_repository::ActivityRepository;
    use anyhow::Result;
    use buckpal_application::application::port::outgoing::load_account_port::LoadAccountPort;
    use buckpal_application::domain::account::AccountId;
    use chrono::{DateTime, NaiveDate, Utc};
    use rusty_money::{money, Money};
    use sqlx::postgres::{PgPool, PgPoolOptions};

    #[async_std::test]
    async fn load_account() -> Result<()> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or(String::from("postgres://localhost/buckpal_test"));

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        let date =
            DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2018, 8, 10).and_hms(0, 0, 0), Utc);

        // setup db
        let first_account_id = given_an_account(&pool).await.unwrap();
        let second_account_id = given_an_account(&pool).await.unwrap();

        let activity_ids =
            given_some_activites_for_account_ids(first_account_id, second_account_id, &pool)
                .await
                .unwrap();
        // end setup db

        let account_id = AccountId(first_account_id);

        let adapter = AccountPersistenceAdapter::new(pool.clone());

        let account = adapter.load_account(&account_id, &date).await.unwrap();

        // cleanup db
        delete_account_with_id(first_account_id, &pool)
            .await
            .unwrap();
        delete_account_with_id(second_account_id, &pool)
            .await
            .unwrap();

        delete_activites_for_ids(activity_ids, &pool).await.unwrap();
        // end cleanup db

        assert_eq!(account.id, Some(account_id));
        assert_eq!(account.activity_window.activities.len(), 2);
        assert_eq!(account.calculate_balance(), money!(500, "AUD"));

        Ok(())
    }

    #[async_std::test]
    async fn updates_activities() {
        use super::AccountPersistenceAdapter;
        use buckpal_application::application::port::outgoing::update_account_state_port::UpdateAccountStatePort;
        use buckpal_application::domain::account::account_test_data::AccountBuilder;
        use buckpal_application::domain::activity::activity_test_data::ActivityBuilder;
        use buckpal_application::domain::activity_window::ActivityWindow;

        let activity_window = ActivityWindow::new(vec![ActivityBuilder::default_activity()
            .with_money(&money!(1, "AUD"))
            .build()]);
        let account = AccountBuilder::default_account()
            .with_baseline_balance(&money!(555, "AUD"))
            .with_activity_window(&activity_window)
            .build();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or(String::from("postgres://localhost/buckpal_test"));

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        let adapter = AccountPersistenceAdapter::new(pool.clone());
        let updated_activities = adapter.update_activities(&account).await.unwrap();

        let activity_ids: Vec<i32> = updated_activities
            .iter()
            .map(|activity| {
                let cloned = activity.id.clone();
                cloned.unwrap().0
            })
            .collect();

        let first_id = activity_ids.first().unwrap();

        let saved_activity = find_activity(*first_id, &pool).await.unwrap();

        delete_activites_for_ids(activity_ids, &pool).await.unwrap();

        assert_eq!(updated_activities.len(), 1);
        assert_eq!(saved_activity.amount, 1);
    }

    async fn find_activity(activity_id: i32, pool: &PgPool) -> Result<ActivityEntity> {
        let entity = sqlx::query!(
            r#"
                SELECT 
                        id,
                        timestamp,
                        owner_account_id,
                        source_account_id,
                        target_account_id,
                        amount
                FROM 
                        activity
                WHERE 
                        id = $1
            "#,
            activity_id,
        )
        .fetch_one(pool)
        .await?;

        let entity = ActivityEntity::new(
            Some(entity.id),
            entity.timestamp,
            entity.owner_account_id,
            entity.source_account_id,
            entity.target_account_id,
            entity.amount,
        );

        Ok(entity)
    }

    async fn given_an_account(pool: &PgPool) -> Result<i32> {
        let entity = sqlx::query!(
            r#"
                INSERT INTO account DEFAULT VALUES RETURNING id 
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(entity.id)
    }

    async fn delete_account_with_id(account_id: i32, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM account WHERE id = $1 
            "#,
            account_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn given_some_activites_for_account_ids(
        first_account_id: i32,
        second_account_id: i32,
        pool: &PgPool,
    ) -> Result<Vec<i32>> {
        let activity_repostiory = ActivityRepository::new(pool.clone());

        let first = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2018, 8, 8).and_hms(8, 0, 0), Utc),
                first_account_id,
                first_account_id,
                second_account_id,
                500,
            ))
            .await?;

        let second = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2018, 8, 8).and_hms(8, 0, 0), Utc),
                second_account_id,
                first_account_id,
                second_account_id,
                500,
            ))
            .await?;

        let third = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2018, 8, 9).and_hms(10, 0, 0), Utc),
                first_account_id,
                second_account_id,
                first_account_id,
                1000,
            ))
            .await?;

        let fourth = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2018, 8, 9).and_hms(10, 0, 0), Utc),
                second_account_id,
                second_account_id,
                first_account_id,
                1000,
            ))
            .await?;

        let fifth = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 9).and_hms(9, 0, 0), Utc),
                first_account_id,
                first_account_id,
                second_account_id,
                1000,
            ))
            .await?;

        let sixth = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 9).and_hms(9, 0, 0), Utc),
                second_account_id,
                first_account_id,
                second_account_id,
                1000,
            ))
            .await?;

        let seventh = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 9).and_hms(10, 0, 0), Utc),
                first_account_id,
                second_account_id,
                first_account_id,
                1000,
            ))
            .await?;

        let eigth = activity_repostiory
            .save(&ActivityEntity::new(
                None,
                DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2019, 8, 9).and_hms(10, 0, 0), Utc),
                second_account_id,
                second_account_id,
                first_account_id,
                1000,
            ))
            .await?;

        Ok(vec![
            first.id.unwrap(),
            second.id.unwrap(),
            third.id.unwrap(),
            fourth.id.unwrap(),
            fifth.id.unwrap(),
            sixth.id.unwrap(),
            seventh.id.unwrap(),
            eigth.id.unwrap(),
        ])
    }

    async fn delete_activites_for_ids(activity_ids: Vec<i32>, pool: &PgPool) -> Result<()> {
        for activity_id in activity_ids {
            sqlx::query!(
                r#"
                DELETE FROM activity WHERE id = $1
            "#,
                activity_id,
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}
