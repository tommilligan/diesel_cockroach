//! Literal `UPSERT` SQL queries.

use diesel::{
    pg::{Pg, PgConnection},
    query_builder::{AstPass, QueryFragment, QueryId},
    query_dsl::RunQueryDsl,
    query_source::QuerySource,
    result::QueryResult,
    Insertable,
};

pub fn upsert_into<T>(target: T) -> IncompleteUpsertStatement<T> {
    IncompleteUpsertStatement::new(target)
}

/// The structure returned by [`upsert_into`].
///
/// The provided methods [`values`] and [`default_values`] will upsert
/// data into the targeted table.
///
/// [`upsert_into`]: ../fn.upsert_into.html
/// [`values`]: #method.values
/// [`default_values`]: #method.default_values
#[derive(Debug, Clone, Copy)]
#[must_use = "Queries are only executed when calling `load`, `get_result` or similar."]
pub struct IncompleteUpsertStatement<T> {
    target: T,
}

impl<T> IncompleteUpsertStatement<T> {
    pub(crate) fn new(target: T) -> Self {
        IncompleteUpsertStatement { target }
    }

    /// Upserts the given values into the table passed to `upsert_into`.
    ///
    /// See the documentation of [`upsert_into`] for
    /// usage examples.
    ///
    /// This method can sometimes produce extremely opaque error messages due to
    /// limitations of the Rust language. If you receive an error about
    /// "overflow evaluating requirement" as a result of calling this method,
    /// you may need an `&` in front of the argument to this method.
    ///
    /// [`upsert_into`]: ../fn.upsert_into.html
    pub fn values<U>(self, records: U) -> UpsertStatement<T, U::Values>
    where
        U: Insertable<T>,
    {
        UpsertStatement::new(self.target, records.values())
    }
}

#[derive(Debug, Copy, Clone)]
#[must_use = "Queries are only executed when calling `load`, `get_result` or similar."]
/// A fully constructed upsert statement.
///
/// The parameters of this struct represent:
///
/// - `T`: The table we are upserting into
/// - `U`: The data being upserted
///
/// See the [CockroachDB docs]. Currently, only the simplest upsert case is supported:
///
/// ```sql
/// UPSERT INTO t (a, b, c) VALUES (1, 2, 3);
/// ```
///
/// [CockroachDB docs]: https://www.cockroachlabs.com/docs/stable/upsert.html
pub struct UpsertStatement<T, U> {
    target: T,
    records: U,
}

impl<T, U> UpsertStatement<T, U> {
    fn new(target: T, records: U) -> Self {
        UpsertStatement { target, records }
    }
}

impl<T, U> QueryFragment<Pg> for UpsertStatement<T, U>
where
    T: QuerySource,
    T::FromClause: QueryFragment<Pg>,
    U: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();

        out.push_sql("UPSERT INTO ");
        self.target.from_clause().walk_ast(out.reborrow())?;
        out.push_sql(" ");
        self.records.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<T, U> RunQueryDsl<PgConnection> for UpsertStatement<T, U> {}

impl<T, U> QueryId for UpsertStatement<T, U> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    table! {
        books (id) {
            id -> Bytea,
            title -> Text,
            page_count -> Int8,
        }
    }

    #[derive(Debug, Clone, PartialEq, Insertable, Queryable)]
    #[table_name = "books"]
    struct Book {
        pub id: Vec<u8>,
        pub title: String,
        pub page_count: i64,
    }

    #[test]
    fn empty() {
        let books: Vec<Book> = Vec::new();
        assert_eq!(
            diesel::debug_query(&upsert_into(books::table).values(&books)).to_string(),
            r#"UPSERT INTO "books"  -- binds: []"#
        );
    }

    #[test]
    fn single() {
        let books = vec![Book {
            id: [0; 1].to_vec(),
            title: "Guards! Guards!".to_owned(),
            page_count: 42,
        }];
        assert_eq!(
            diesel::debug_query(&upsert_into(books::table).values(&books)).to_string(),
            r#"UPSERT INTO "books" ("id", "title", "page_count") VALUES ($1, $2, $3) -- binds: [[0], "Guards! Guards!", 42]"#
        );
    }

    #[test]
    fn many() {
        let books = vec![
            Book {
                id: [0; 1].to_vec(),
                title: "Guards! Guards!".to_owned(),
                page_count: 42,
            },
            Book {
                id: [1; 8].to_vec(),
                title: "Shift".to_owned(),
                page_count: i64::MAX,
            },
        ];
        assert_eq!(
            diesel::debug_query(&upsert_into(books::table).values(&books)).to_string(),
            r#"UPSERT INTO "books" ("id", "title", "page_count") VALUES ($1, $2, $3), ($4, $5, $6) -- binds: [[0], "Guards! Guards!", 42, [1, 1, 1, 1, 1, 1, 1, 1], "Shift", 9223372036854775807]"#
        );
    }
}
