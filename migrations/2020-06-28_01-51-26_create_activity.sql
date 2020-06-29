CREATE TABLE IF NOT EXISTS activity (
    id                  SERIAL PRIMARY KEY,
    timestamp           TIMESTAMPTZ NOT NULL,
    owner_account_id    INT NOT NULL,
    source_account_id   INT NOT NULL,
    target_account_id   INT NOT NULL,
    amount              BIGINT NOT NULL
);

