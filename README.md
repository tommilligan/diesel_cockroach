# diesel_cockroach

[![Crates.io](https://img.shields.io/crates/v/diesel_cockroach)](https://crates.io/crates/diesel_cockroach)
[![CircleCI branch](https://img.shields.io/circleci/project/github/tommilligan/diesel_cockroach/master.svg)](https://circleci.com/gh/tommilligan/diesel_cockroach)
[![GitHub](https://img.shields.io/github/license/tommilligan/diesel_cockroach)](https://github.com/tommilligan/diesel_cockroach/blob/master/LICENSE)

Additional [Diesel](https://diesel.rs/) ORM support for [CockroachDB syntax](https://www.cockroachlabs.com/docs/stable/sql-statements.html).

## Installation

```
cargo install diesel_cockroach
```

## Feature Support

Currently supported features are listed below:

- [x] [`UPSERT`](https://www.cockroachlabs.com/docs/stable/upsert.html)

## Usage

See the official [`insert_into` documentation](https://docs.diesel.rs/diesel/fn.insert_into.html) for general examples.

Just replace the `diesel::insert_into` function with the disired function from `diesel_cockroach`:

```rust
use diesel_cockroach::upsert::upsert_into;

let new_users = vec![
    name.eq("Tess"),
    name.eq("Jim"),
];

let rows_upserted = upsert_into(users)
    .values(&new_users)
    .execute(&connection);

assert_eq!(Ok(2), rows_upserted);
```
