use crate::application::port::incoming::send_money_use_case::{SendMoneyCommand, SendMoneyUseCase};
use crate::application::port::outgoing::{
    account_lock::AccountLock, load_account_port::LoadAccountPort,
    update_account_state_port::UpdateAccountStatePort,
};
use crate::application::service::error::ServiceError;
use crate::application::service::money_transfer_properties::MoneyTransferProperties;
use anyhow::{anyhow, Result};

pub struct SendMoneyService<'a> {
    load_account_port: Box<dyn LoadAccountPort + 'a>,
    account_lock: Box<dyn AccountLock + 'a>,
    update_account_state_port: Box<dyn UpdateAccountStatePort + 'a>,
    money_transfer_properties: MoneyTransferProperties,
}

impl<'a> SendMoneyService<'a> {
    pub fn new(
        load_account_port: Box<dyn LoadAccountPort + 'a>,
        account_lock: Box<dyn AccountLock + 'a>,
        update_account_state_port: Box<dyn UpdateAccountStatePort + 'a>,
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

impl<'a> SendMoneyUseCase for SendMoneyService<'a> {
    fn send_money(&self, command: &SendMoneyCommand) -> bool {
        use chrono::{Duration, Utc};

        if self.check_threshold(command).is_err() {
            return false;
        }

        let baseline_date = Utc::now() - Duration::days(10);

        let mut source_account = self
            .load_account_port
            .load_account(&command.source_account_id, &baseline_date);

        let mut target_account = self
            .load_account_port
            .load_account(&command.target_account_id, &baseline_date);

        let source_account_id = source_account
            .clone()
            .id
            .expect("expected source account ID not to be empty");
        let target_account_id = target_account
            .clone()
            .id
            .expect("expected target account ID not to be empty");

        self.account_lock.lock_account(&source_account_id);
        if !source_account.withdraw(&command.money, &target_account_id) {
            self.account_lock.release_account(&source_account_id);
            return false;
        }

        self.account_lock.lock_account(&target_account_id);
        if !target_account.deposit(&command.money, &source_account_id) {
            self.account_lock.release_account(&source_account_id);
            self.account_lock.release_account(&target_account_id);
            return false;
        }

        self.update_account_state_port
            .update_activities(&source_account);
        self.update_account_state_port
            .update_activities(&target_account);

        self.account_lock.release_account(&source_account_id);
        self.account_lock.release_account(&target_account_id);

        true
    }
}

impl<'a> SendMoneyService<'a> {
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
        account_lock::MockAccountLock, load_account_port::MockLoadAccountPort,
        update_account_state_port::MockUpdateAccountStatePort,
    };
    use crate::application::service::money_transfer_properties::MoneyTransferProperties;
    use crate::domain::account::account_test_data::AccountBuilder;
    use crate::domain::account::{Account, AccountId};
    use mockall::*;
    use mocktopus::mocking::*;
    use rusty_money::{money, Money};

    #[test]
    fn given_withdrawal_fails_then_only_source_account_is_locked_and_released() {
        let mut load_account_port = MockLoadAccountPort::new();
        let mut account_lock = MockAccountLock::new();
        let update_account_state_port = MockUpdateAccountStatePort::new();
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

        let success = send_money_service.send_money(&command);
        assert_eq!(success, false);
    }

    #[test]
    fn transation_succeeds() {
        let mut load_account_port = MockLoadAccountPort::new();
        let mut account_lock = MockAccountLock::new();
        let mut update_account_state_port = MockUpdateAccountStatePort::new();
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

        let success = send_money_service.send_money(&command);
        assert_eq!(success, true);
    }

    fn then_accounts_have_been_updated(
        account_ids: Vec<&AccountId>,
        update_account_state_port_mock: &mut MockUpdateAccountStatePort,
    ) {
        let mut cloned = vec![];
        for account_id in account_ids.clone() {
            cloned.push(account_id.clone());
        }

        update_account_state_port_mock
            .expect_update_activities()
            .withf(move |account| cloned.contains(&&account.clone().id.take().unwrap()))
            .times(account_ids.len())
            .returning(|_| ());
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

        let cloned = source_account.clone();
        load_account_port_mock
            .expect_load_account()
            .with(predicate::eq(id.clone()), predicate::always())
            .returning(move |_account_id, _baseline_date| source_account.clone());

        cloned
    }

    fn given_withdrawal_will_succeed(account: &Account) {
        let cloned = account.clone();
        Account::withdraw.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(true)
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }

    fn given_withdrawal_will_fail(account: &Account) {
        let cloned = account.clone();
        Account::withdraw.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(false)
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }

    fn given_deposit_will_succeed(account: &Account) {
        let cloned = account.clone();
        Account::deposit.mock_safe(move |curr, money, target| {
            if curr.id == cloned.id {
                MockResult::Return(true)
            } else {
                MockResult::Continue((curr, money, target))
            }
        })
    }
}
