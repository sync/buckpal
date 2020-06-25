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

impl<'a> SendMoneyUseCase for SendMoneyService<'a> {
    fn send_money(&self, command: &SendMoneyCommand) -> bool {
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
