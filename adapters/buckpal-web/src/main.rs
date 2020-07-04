#[macro_use]
extern crate log;

mod utils;

use crate::utils::success_to_res;
use anyhow::Result;
use buckpal_application::application::port::incoming::send_money_use_case::{
    SendMoneyCommand, SendMoneyUseCase,
};
use buckpal_application::application::service::{
    money_transfer_properties::MoneyTransferProperties, no_op_account_lock::NoOpAccountLock,
    send_money_service::SendMoneyService,
};
use buckpal_application::domain::account::AccountId;
use buckpal_persistence::account_persistence_adapter::AccountPersistenceAdapter;
use rusty_money::{money, Money};
use sqlx::postgres::PgPool;
use std::env;
use std::sync::Arc;
use tide::{security::CorsMiddleware, Error, Request, Response, Server, StatusCode};

struct AppState {
    send_money_use_case: Arc<dyn SendMoneyUseCase + Send + Sync>,
}

impl AppState {
    fn new(send_money_use_case: Arc<dyn SendMoneyUseCase + Send + Sync>) -> Self {
        Self {
            send_money_use_case,
        }
    }
}

fn validate_accounts_send_params(req: &Request<AppState>) -> tide::Result<(i32, i32, i64)> {
    let source_account_id: i32 =
        req.param("sourceAccountId")
            .map_err(|err: std::num::ParseIntError| {
                Error::from_str(
                    StatusCode::UnprocessableEntity,
                    format!("Invalid sourceAccountId: {}", err.to_string()),
                )
            })?;

    let target_account_id: i32 =
        req.param("targetAccountId")
            .map_err(|err: std::num::ParseIntError| {
                Error::from_str(
                    StatusCode::UnprocessableEntity,
                    format!("Invalid targetAccountId: {}", err.to_string()),
                )
            })?;

    let amount: i64 = req
        .param("amount")
        .map_err(|err: std::num::ParseIntError| {
            Error::from_str(
                StatusCode::UnprocessableEntity,
                format!("Invalid amount: {}", err.to_string()),
            )
        })?;

    Ok((source_account_id, target_account_id, amount))
}

async fn handle_accounts_send(req: Request<AppState>) -> tide::Result<Response> {
    let (source_account_id, target_account_id, amount) = validate_accounts_send_params(&req)?;

    let command = SendMoneyCommand::new(
        AccountId(source_account_id),
        AccountId(target_account_id),
        money!(amount, "AUD"),
    );

    let send_money_use_case = req.state().send_money_use_case.clone();

    send_money_use_case
        .send_money(&command)
        .await
        .map_err(|err| Error::from_str(StatusCode::BadRequest, err.to_string()))?;

    success_to_res("Money Sent!")
}

fn main() -> Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let database_url = env::var("DATABASE_URL")?;
    let port = env::var("PORT").unwrap_or_else(|_| String::from("6000"));
    let listen_addr = format!("0.0.0.0:{}", port);

    smol::block_on(async {
        let pool = PgPool::builder()
            .max_size(5) // maximum number of connections in the pool
            .build(&database_url)
            .await?;

        let account_persistence_adapter = AccountPersistenceAdapter::new(pool);
        let no_op_account_lock = NoOpAccountLock::default();
        let money_transfer_properties = MoneyTransferProperties::new();
        let send_money_use_case = SendMoneyService::new(
            Box::new(account_persistence_adapter.clone()),
            Box::new(no_op_account_lock),
            Box::new(account_persistence_adapter),
            money_transfer_properties,
        );

        let app_state = AppState::new(Arc::new(send_money_use_case));

        let mut app = Server::with_state(app_state);

        let cors = CorsMiddleware::new();

        app.middleware(cors);

        app.at("/accounts/send/:sourceAccountId/:targetAccountId/:amount")
            .post(handle_accounts_send);

        info!("Starting at: {}", listen_addr);

        app.listen(listen_addr).await?;

        Ok(())
    })
}
