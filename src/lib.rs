//! Adds support for the CockroachDB specific SQL queries to Diesel.

#[cfg(test)]
#[macro_use]
extern crate diesel;

pub mod as_of_system_time;
pub mod upsert;
