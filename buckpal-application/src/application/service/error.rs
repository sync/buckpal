use rusty_money::Money;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Maximum threshold for transferring money exceeded: tried to transfer {threshold:?} but threshold is {actual:?}!")]
    ThresholdExceededException { threshold: Money, actual: Money },
    #[error("May withdraw failed with the following balance: `{0}`")]
    MayWithdrawFailed(i64),
}
