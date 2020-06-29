# Example Implementation of a Hexagonal Architecture in Rust

This is the companion code to the eBook [Get Your Hands Dirty on Clean Architecture](https://leanpub.com/get-your-hands-dirty-on-clean-architecture).

It implements a domain-centric "Hexagonal" approach of a common web application with Rust. 

## Companion Articles

* [Hexagonal Architecture with Java and Spring](https://reflectoring.io/spring-hexagonal/)
* [Building a Multi-Module Spring Boot Application with Gradle](https://reflectoring.io/spring-boot-gradle-multi-module/)

## Prerequisites

* Rust Nightly (needed for our test mocks on Struct functions)

## Rust nightly

```sh
rustup toolchain install nightly

cd buckpal;
rustup override set nightly
```

## Postgres

```sh
# run docker locally
# and tell to create a buckpal user / database
docker run --name buckpal-db \
  -e POSTGRES_PASSWORD=buckpal \
  -e POSTGRES_USER=buckpal \
  -e POSTGRES_DB=buckpal \
  -v buckpal-db-data:/var/lib/postgresql/data \
  -p 5432:5432 -d postgres:12-alpine

# subsequent time you just have to restart the image
docker restart buckpal-db
```
## Migrations

```sh
cargo install sqlx-cli --git https://github.com/launchbadge/sqlx.git --no-default-features --features postgres

# make sure database is created then
sqlx migrate run

# to create a new migration
sqlx migrate add your_migration_name
sqlx migrate run

#also create test database
createdb meet_test
DATABASE_URL=postgres://buckpal:buckpal@localhost/buckpal_test sqlx migrate run
```
