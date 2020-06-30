use crate::application::port::incoming::send_money_use_case::{SendMoneyCommand, SendMoneyUseCase};
use crate::application::port::outgoing::{
    account_lock::AccountLock, load_account_port::LoadAccountPort,
    update_account_state_port::UpdateAccountStatePort,
};
use crate::application::service::error::ServiceError;
use crate::application::service::money_transfer_properties::MoneyTransferProperties;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

pub struct SendMoneyService {
    load_account_port: Box<dyn LoadAccountPort + Send + Sync>,
    account_lock: Box<dyn AccountLock + Send + Sync>,
    update_account_state_port: Box<dyn UpdateAccountStatePort + Send + Sync>,
    money_transfer_properties: MoneyTransferProperties,
}

impl SendMoneyService {
    pub fn new(
        load_account_port: Box<dyn LoadAccountPort + Send + Sync>,
        account_lock: Box<dyn AccountLock + Send + Sync>,
        update_account_state_port: Box<dyn UpdateAccountStatePort + Send + Sync>,
        money_transfer_properties: MoneyTransferProperties,
    ) -> Self {
        Self {
            load_account_port,
            account_lock,
            update_account_state_port,
            money_transfer_properties,
        }
    }
}

#[async_trait]
impl SendMoneyUseCase for SendMoneyService {
    async fn send_money(&self, command: &SendMoneyCommand) -> Result<()> {
        use chrono::{Duration, Utc};

        if let Err(err) = self.check_threshold(command) {
            return Err(err);
        }

        let baseline_date = Utc::now() - Duration::days(10);

        let mut source_account = self
            .load_account_port
            .load_account(&command.source_account_id, &baseline_date)
            .await?;

        let mut target_account = self
            .load_account_port
            .load_account(&command.target_account_id, &baseline_date)
            .await?;

        let source_account_id = source_account
            .clone()
            .id
            .expect("expected source account ID not to be empty");
        let target_account_id = target_account
            .clone()
            .id
            .expect("expected target account ID not to be empty");

        self.account_lock.lock_account(&source_account_id);
        if let Err(err) = source_account.withdraw(&command.money, &target_account_id) {
            self.account_lock.release_account(&source_account_id);
            return Err(err);
        }

        self.account_lock.lock_account(&target_account_id);
        if let Err(err) = target_account.deposit(&command.money, &source_account_id) {
            self.account_lock.release_account(&source_account_id);
            self.account_lock.release_account(&target_account_id);
            return Err(err);
        }

        self.update_account_state_port
            .update_activities(&source_account)
            .await?;
        self.update_account_state_port
            .update_activities(&target_account)
            .await?;

        self.account_lock.release_account(&source_account_id);
        self.account_lock.release_account(&target_account_id);

        Ok(())
    }
}

impl SendMoneyService {
    fn check_threshold(&self, command: &SendMoneyCommand) -> Result<()> {
        if command.money > self.money_transfer_properties.maximum_transfer_threshold() {
            let error = ServiceError::ThresholdExceededException {
                threshold: self.money_transfer_properties.maximum_transfer_threshold(),
                actual: command.money.clone(),
            };
            return Err(anyhow!(error));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SendMoneyService;
    use crate::application::port::incoming::send_money_use_case::{
        SendMoneyCommand, SendMoneyUseCase,
    };
    use crate::application::port::outgoing::{
        account_lock::MockAccountLock, load_account_port::LoadAccountPort,
        update_account_state_port::UpdateAccountStatePort,
    };
    use crate::application::service::money_transfer_properties::MoneyTransferProperties;
    use crate::domain::account::account_test_data::AccountBuilder;
    use crate::domain::account::{Account, AccountId};
    use crate::domain::activity::Activity;
    use anyhow::anyhow;
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use mockall::*;
    use mocktopus::mocking::*;
    use rusty_money::{money, Money};

    #[async_std::test]
    async fn given_withdrawal_fails_then_only_source_account_is_locked_and_released() {
        let mut load_account_port = MockLoadAccountPort::new();
        let mut account_lock = MockAccountLock::new();
        let update_account_state_port = MockUpdateAccountStatePort::default();
        let money_transfer_properties = MoneyTransferProperties::default();

        let source_account_id = AccountId(41);
        let source_account = given_an_account_with_id(&source_account_id, &mut load_account_port);

        let target_account_id = AccountId(42);
        let target_account = given_an_account_with_id(&target_account_id, &mut load_account_port);

        given_withdrawal_will_fail(&source_account);
        given_deposit_will_succeed(&target_account);

        account_lock
            .expect_lock_account()
            .with(predicate::eq(source_account_id.clone()))
            .times(1)
            .returning(|_| ());

        account_lock
            .expect_release_account()
            .with(predicate::eq(source_account_id.clone()))
            .times(1)
            .returning(|_| ());

        account_lock
            .expect_lock_account()
            .with(predicate::eq(target_account_id.clone()))
            .times(0);

        let command =
            SendMoneyCommand::new(source_account_id, target_account_id, money!(300, "AUD"));

        let send_money_service = SendMoneyService::new(
            Box::new(load_account_port),
            Box::new(account_lock),
            Box::new(update_account_state_port),
            money_transfer_properties,
        );

        let success = send_money_service.send_money(&command).await.is_ok();
        assert_eq!(success, false);
    }

    #[async_std::test]
    async fn transation_succeeds() {
        let mut load_account_port = MockLoadAccountPort::new();
        let mut account_lock = MockAccountLock::new();
        let mut update_account_state_port = MockUpdateAccountStatePort::default();
        let money_transfer_properties = MoneyTransferProperties::default();

        let source_account = given_source_account(&mut load_account_port);
        let source_account_id = source_account.clone().id.unwrap();

        let target_account = given_target_account(&mut load_account_port);
        let target_account_id = source_account.clone().id.unwrap();

        given_withdrawal_will_succeed(&source_account);
        given_deposit_will_succeed(&target_account);

        let money = money!(500, "AUD");

        account_lock
            .expect_lock_account()
            .with(predicate::eq(source_account_id.clone()))
            .returning(|_| ());

        account_lock
            .expect_release_account()
            .with(predicate::eq(source_account_id.clone()))
            .returning(|_| ());

        account_lock
            .expect_lock_account()
            .with(predicate::eq(target_account_id.clone()))
            .returning(|_| ());

        account_lock
            .expect_release_account()
            .with(predicate::eq(target_account_id.clone()))
            .returning(|_| ());

        then_accounts_have_been_updated(
            vec![&source_account_id, &target_account_id],
            &mut update_account_state_port,
        );

        let command = SendMoneyCommand::new(source_account_id, target_account_id, money);

        let send_money_service = SendMoneyService::new(
            Box::new(load_account_port),
            Box::new(account_lock),
            Box::new(update_account_state_port),
            money_transfer_properties,
        );

        let success = send_money_service.send_money(&command).await.is_ok();
        assert_eq!(success, true);
    }

    fn then_accounts_have_been_updated(
        account_ids: Vec<&AccountId>,
        update_account_state_port_mock: &mut MockUpdateAccountStatePort,
    ) {
        update_account_state_port_mock.expect_update_activities(account_ids);
    }

    fn given_target_account(load_account_port_mock: &mut MockLoadAccountPort) -> Account {
        given_an_account_with_id(&AccountId(42), load_account_port_mock)
    }

    fn given_source_account(load_account_port_mock: &mut MockLoadAccountPort) -> Account {
        given_an_account_with_id(&AccountId(41), load_account_port_mock)
    }

    fn given_an_account_with_id(
        id: &AccountId,
        load_account_port_mock: &mut MockLoadAccountPort,
    ) -> Account {
        let source_account = AccountBuilder::default_account()
            .with_account_id(&id)
            .build();

        load_account_port_mock.expect_load_account(&source_account);

        source_account
    }

    fn given_withdrawal_will_succeed(account: &Account) {
        let cloned = account.clone();
        Account::withdraw.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(Ok(()))
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }

    fn given_withdrawal_will_fail(account: &Account) {
        let cloned = account.clone();
        Account::withdraw.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(Err(anyhow!("Something bad happened")))
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }

    fn given_deposit_will_succeed(account: &Account) {
        let cloned = account.clone();
        Account::deposit.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(Ok(()))
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }

    #[derive(Debug, Default)]
    struct MockUpdateAccountStatePort {}

    impl MockUpdateAccountStatePort {
        fn expect_update_activities(&mut self, _account_ids: Vec<&AccountId>) {
            // here eventually need to check that update_activities  got call with those ids
        }
    }

    #[async_trait]
    impl UpdateAccountStatePort for MockUpdateAccountStatePort {
        async fn update_activities(&self, _account: &Account) -> Result<Vec<Activity>> {
            // do nothing here
            Ok(vec![])
        }
    }

    #[derive(Debug)]
    struct MockLoadAccountPort {
        available_accounts: Vec<Account>,
    }

    impl MockLoadAccountPort {
        fn new() -> Self {
            Self {
                available_accounts: vec![],
            }
        }

        fn expect_load_account(&mut self, account: &Account) {
            self.available_accounts.push(account.clone());
        }
    }

    #[async_trait]
    impl LoadAccountPort for MockLoadAccountPort {
        async fn load_account(
            &self,
            account_id: &AccountId,
            _baseline_date: &DateTime<Utc>,
        ) -> Result<Account> {
            let account = self
                .available_accounts
                .iter()
                .find(|account| match account.id.clone() {
                    Some(id) => id == *account_id,
                    None => false,
                });

            account
                .map(|account| account.clone())
                .ok_or(anyhow!("No matching account found from stub"))
        }
    }
}
