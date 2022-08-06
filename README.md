# sqlx-derive-with
Derive `sqlx::FromRow` specific to the given database.

sqlx-derive-with supports `decode` attribute to use custom decoder function to specific columns.
This feature is not (and unable to be, I think) supported by upstream `sqlx::FromRow`.

## Usage
```rust
use sqlx::Connection as _;

#[derive(sqlx_derive_with::FromRow)]
#[sqlx_with(db = "sqlx::Sqlite")]
struct Row {
    #[sqlx_with(decode = "split_x")]
    x: (i64, i64),
    y: String,
}

fn split_x(index: &str, row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<(i64, i64)> {
    use sqlx::Row as _;
    let n: i64 = row.try_get(index)?;
    Ok((n, n + 2))
}

#[tokio::main]
async fn main() {
    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as x, 'hello' as y")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.x, (10, 12));
    assert_eq!(row.y, "hello");
}
```
