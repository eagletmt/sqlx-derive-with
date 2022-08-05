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

#[tokio::test]
async fn rename_all_snake_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "snake_case")]
    #[allow(non_snake_case)]
    struct Row {
        FooBar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foo_bar, 20 as FooBar")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.FooBar, 10);
}

#[tokio::test]
async fn rename_all_lower_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "lowercase")]
    #[allow(non_snake_case)]
    struct Row {
        FooBar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foobar, 20 as FooBar")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.FooBar, 10);
}

#[tokio::test]
async fn rename_all_upper_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "UPPERCASE")]
    struct Row {
        foobar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foobar, 20 as FOOBAR")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foobar, 20);
}

#[tokio::test]
async fn rename_all_camel_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "camelCase")]
    struct Row {
        foo_bar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foo_bar, 20 as fooBar")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foo_bar, 20);
}

#[tokio::test]
async fn rename_all_pascal_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "PascalCase")]
    struct Row {
        foo_bar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foo_bar, 20 as fooBar, 30 as FooBar")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foo_bar, 30);
}

#[tokio::test]
async fn rename_all_screaming_snake_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "SCREAMING_SNAKE_CASE")]
    struct Row {
        foo_bar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as("select 10 as foo_bar, 20 as FooBar, 30 as FOO_BAR")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foo_bar, 30);
}

#[tokio::test]
async fn rename_all_kebab_case() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "kebab-case")]
    struct Row {
        foo_bar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as(r#"select 10 as foo_bar, 20 as "foo-bar""#)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foo_bar, 20);
}

#[tokio::test]
async fn rename_all_precedence() {
    #[derive(sqlx_with::FromRow)]
    #[sqlx_with(db = "sqlx::Sqlite", rename_all = "kebab-case")]
    struct Row {
        #[sqlx_with(rename = "hi")]
        foo_bar: i64,
    }

    let mut conn = sqlx::SqliteConnection::connect(":memory:").await.unwrap();
    let row: Row = sqlx::query_as(r#"select 10 as foo_bar, 20 as "foo-bar", 30 as hi"#)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert_eq!(row.foo_bar, 30);
}
