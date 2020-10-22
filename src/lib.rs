//! Adds support for the CockroachDB specific SQL queries to Diesel.

#[cfg(test)]
#[macro_use]
extern crate diesel;

pub mod upsert;
