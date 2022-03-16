//! Support for the `AS OF SYSTEM TIME` SQL expression.
//!
//! # Example
//!
//! ```rust
//! #[macro_use] extern crate diesel;
//! use diesel::pg::{data_types::{PgInterval, PgTimestamp}, Pg};
//! use diesel::prelude::*;
//! use diesel_cockroach::as_of_system_time::*;
//!
//! table! {
//!     books (id) {
//!         id -> Bytea,
//!     }
//! }
//!
//! // Since CockroachDB v20.2.
//! assert_eq!(
//!   diesel::debug_query::<Pg, _>(
//!     &books::table
//!       .select(books::dsl::id)
//!       .as_of_system_time(follower_read_timestamp)
//!   )
//!   .to_string(),
//!   r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME follower_read_timestamp() -- binds: []"#
//! );
//!
//! // Since CockroachDB v21.2
//! assert_eq!(
//!   diesel::debug_query::<Pg, _>(
//!     &books::table
//!       .select(books::dsl::id)
//!       .as_of_system_time(with_min_timestamp(PgTimestamp(0)))
//!   )
//!   .to_string(),
//!   r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_min_timestamp($1) -- binds: [PgTimestamp(0)]"#
//! );
//!
//! // Since CockroachDB v21.2
//! assert_eq!(
//!   diesel::debug_query::<Pg, _>(
//!     &books::table
//!       .select(books::dsl::id)
//!       .as_of_system_time(with_max_staleness(PgInterval::from_microseconds(0)))
//!   )
//!   .to_string(),
//!   r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_max_staleness($1) -- binds: [PgInterval { microseconds: 0, days: 0, months: 0 }]"#
//! );
//! ```
//!
//! See the [CockroachDB docs].
//!
//! [CockroachDB docs]: https://www.cockroachlabs.com/docs/stable/as-of-system-time.html

use diesel::expression::Expression;
use diesel::pg::{types::sql_types::Timestamptz, Pg};
use diesel::query_builder::{AstPass, QueryFragment, SelectQuery};
use diesel::sql_types::{Bool, Interval};
use diesel::{no_arg_sql_function, sql_function};
use diesel::{DieselNumericOps, QueryId, QueryResult};

// Module does not build without these macros
use diesel::{
    __diesel_parse_type_args, __diesel_sql_function_body, __diesel_sqlite_register_fn,
    no_arg_sql_function_body, no_arg_sql_function_body_except_to_sql, static_cond,
};

/// Represents the return type of `.as_of_system_time(system_time)`.
///
/// The parameters of this struct represent:
///
/// - `S`: The source select query
/// - `T`: The system time
pub struct AsOfSystemTime<S, T> {
    source: S,
    system_time: T,
}

impl<S, T> AsOfSystemTime<S, T> {
    fn new(source: S, system_time: T) -> Self {
        AsOfSystemTime {
            source,
            system_time,
        }
    }
}

impl<S, T, ST> QueryFragment<Pg> for AsOfSystemTime<S, T>
where
    S: SelectQuery<SqlType = ST> + QueryFragment<Pg>,
    T: Expression + QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        self.source.walk_ast(out.reborrow())?;
        out.push_sql(" AS OF SYSTEM TIME ");
        self.system_time.walk_ast(out.reborrow())?;
        Ok(())
    }
}

/// The `as_of_system_time` method.
pub trait AsOfSystemTimeDsl<T> {
    type Output;

    /// Adds the `AS OF SYSTEM TIME` expression to a `SELECT` statement.
    /// 
    /// Since CockroachDB v20.2.
    fn as_of_system_time(self, system_time: T) -> Self::Output;
}

impl<S, T> AsOfSystemTimeDsl<T> for S {
    type Output = AsOfSystemTime<S, T>;

    fn as_of_system_time(self, system_time: T) -> Self::Output {
        AsOfSystemTime::new(self, system_time)
    }
}

no_arg_sql_function!(
    follower_read_timestamp,
    Timestamptz,
    "Represents the SQL function `follower_read_timestamp()`."
);

sql_function! {
    /// Represents the SQL function `with_min_timestamp(TIMESTAMPTZ)`.
    /// 
    /// Since CockroachDB v21.2.
    fn with_min_timestamp(timestamp: Timestamptz) -> Timestamptz;
}

sql_function! {
    /// Represents the SQL function `with_min_timestamp(TIMESTAMPTZ, [nearest_only])`.
    /// 
    /// Since CockroachDB v21.2.
    #[sql_name = "with_min_timestamp"]
    fn with_min_timestamp_nearest_only(
        timestamp: Timestamptz,
        nearest_only: Bool
    ) -> Timestamptz;
}

sql_function! {
    /// Represents the SQL function `with_max_staleness(INTERVAL)`.
    /// 
    /// Since CockroachDB v21.2.
    fn with_max_staleness(interval: Interval) -> Timestamptz;
}

sql_function! {
    /// Represents the SQL function `with_max_staleness(INTERVAL, [nearest_only])`.
    /// 
    /// Since CockroachDB v21.2.
    #[sql_name = "with_max_staleness"]
    fn with_max_staleness_nearest_only(
        interval: Interval,
        nearest_only: Bool
    ) -> Timestamptz;
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::pg::data_types::{PgInterval, PgTimestamp};
    use diesel::prelude::*;
    use pretty_assertions::assert_eq;

    table! {
        books (id) {
            id -> Bytea,
        }
    }

    #[derive(Debug, Clone, PartialEq, Insertable, Queryable)]
    #[table_name = "books"]
    struct Book {
        pub id: Vec<u8>,
    }

    #[test]
    fn select_books_as_of_system_time_follower_read_timestamp() {
        assert_eq!(
            diesel::debug_query::<Pg, _>(
                &books::table
                    .select(books::dsl::id)
                    .as_of_system_time(follower_read_timestamp)
            )
            .to_string(),
            r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME follower_read_timestamp() -- binds: []"#
        );
    }

    #[test]
    fn select_books_as_of_system_time_with_min_timestamp() {
        assert_eq!(
            diesel::debug_query::<Pg, _>(
                &books::table
                    .select(books::dsl::id)
                    .as_of_system_time(with_min_timestamp(PgTimestamp(0)))
            )
            .to_string(),
            r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_min_timestamp($1) -- binds: [PgTimestamp(0)]"#
        );
    }

    #[test]
    fn select_books_as_of_system_time_with_min_timestamp_nearest_only() {
        assert_eq!(
            diesel::debug_query::<Pg, _>(
                &books::table
                    .select(books::dsl::id)
                    .as_of_system_time(with_min_timestamp_nearest_only(PgTimestamp(0), true))
            )
            .to_string(),
            r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_min_timestamp($1, $2) -- binds: [PgTimestamp(0), true]"#
        );
    }

    #[test]
    fn select_books_as_of_system_time_with_max_staleness() {
        assert_eq!(
            diesel::debug_query::<Pg, _>(
                &books::table
                    .select(books::dsl::id)
                    .as_of_system_time(with_max_staleness(PgInterval::from_microseconds(0)))
            )
            .to_string(),
            r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_max_staleness($1) -- binds: [PgInterval { microseconds: 0, days: 0, months: 0 }]"#
        );
    }

    #[test]
    fn select_books_as_of_system_time_with_max_staleness_nearest_only() {
        assert_eq!(
            diesel::debug_query::<Pg, _>(&books::table.select(books::dsl::id).as_of_system_time(
                with_max_staleness_nearest_only(PgInterval::from_microseconds(0), true)
            ))
            .to_string(),
            r#"SELECT "books"."id" FROM "books" AS OF SYSTEM TIME with_max_staleness($1, $2) -- binds: [PgInterval { microseconds: 0, days: 0, months: 0 }, true]"#
        );
    }
}
