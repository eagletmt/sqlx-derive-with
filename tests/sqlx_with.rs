use sqlx::Connection as _;

#[tokio::test]
async fn it_works() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite")]
    struct Row {
        x: i64,
        y: String,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as x, 'hello' as y")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.x, 10);
    assert_eq!(row.y, "hello");
}

#[tokio::test]
async fn rename() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite")]
    struct Row {
        #[sqlx_with(rename = "z")]
        x: i64,
        y: String,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as x, 'hello' as y, 20 as z")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.x, 20);
    assert_eq!(row.y, "hello");
}

#[tokio::test]
async fn default() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite")]
    struct Row {
        #[sqlx_with(rename = "z", default)]
        x: i64,
        y: String,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as x, 'hello' as y")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.x, 0);
    assert_eq!(row.y, "hello");
}

#[tokio::test]
async fn decode() {
    #[derive(sqlx_with::FromRow)]
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

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as x, 'hello' as y")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.x, (10, 12));
    assert_eq!(row.y, "hello");
}
