use rewrite_root::dependency;
use sqlx::sqlite::SqlitePoolOptions;

#[dependency]
async fn test_it() -> Result<(), sqlx::Error> {
    let mut pool = SqlitePoolOptions::connect("sqlite://memory:").await.unwrap();

    sqlx::query("test").execute(&mut pool).await
}

#[dependency]
async fn test_let() -> Result<(), sqlx::Error> {
    let mut pool = SqlitePoolOptions::connect("sqlite://memory:").await.unwrap();

    let res = sqlx::query("test").execute(&mut pool).await;

    res
}


