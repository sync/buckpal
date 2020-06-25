use rusty_money::Money;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Maximum threshold for transferring money exceeded: tried to transfer {threshold:?} but threshold is {actual:?}!")]
    ThresholdExceededException { threshold: Money, actual: Money },
}
