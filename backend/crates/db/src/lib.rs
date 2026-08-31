//! Database access layer: connection pooling, models, and query functions.

pub mod error;
pub mod models;
pub mod queries;

pub use sqlx::MySqlPool;

/// Migrator built from this crate's migrations directory.
///
/// Exported so integration tests can pass `#[sqlx::test(migrator = "db::MIGRATOR")]`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub type Result<T> = std::result::Result<T, error::DbError>;

/// Connects to MySQL and runs any pending migrations.
///
/// If the target database does not yet exist it is created before migrations
/// run. The database name is inferred from the path component of `database_url`.
///
/// # Errors
///
/// Returns [`error::DbError`] if the connection cannot be established, the
/// database cannot be created, or a migration fails.
pub async fn connect(database_url: &str) -> Result<MySqlPool> {
    ensure_database(database_url).await?;

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Creates the target database if it does not already exist.
async fn ensure_database(database_url: &str) -> Result<()> {
    let (server_url, db_name) = split_database_url(database_url);
    if db_name.is_empty() {
        return Ok(());
    }
    let conn = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(server_url)
        .await?;
    let sql = format!(
        "CREATE DATABASE IF NOT EXISTS `{db_name}` \
         CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"
    );
    sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
        .execute(&conn)
        .await?;
    conn.close().await;
    Ok(())
}

/// Splits a MySQL URL into a server level URL and the database name.
///
/// For `mysql://user:pass@host/mydb` this returns
/// `("mysql://user:pass@host/", "mydb")`.
fn split_database_url(database_url: &str) -> (&str, &str) {
    let after_scheme = database_url.find("://").map(|i| i + 3).unwrap_or(0);
    match database_url[after_scheme..].find('/') {
        Some(offset) => {
            let slash = after_scheme + offset;
            (&database_url[..=slash], &database_url[slash + 1..])
        }
        None => (database_url, ""),
    }
}
