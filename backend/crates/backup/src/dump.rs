//! Logical MariaDB dump and restore.
//!
//! A dump is a plain SQL script with statements separated by blank lines. Values are written as
//! hex literals or simple quoted literals, so no statement ever contains a blank line.

use futures_util::TryStreamExt;
use sqlx::{AssertSqlSafe, MySqlConnection, MySqlPool, Row};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use crate::{BackupError, Result};

/// Rows per `INSERT` statement.
const BATCH_ROWS: usize = 500;

const HEADER: &str = "SET NAMES utf8mb4;\n\n\
                      SET time_zone = '+00:00';\n\n\
                      SET FOREIGN_KEY_CHECKS = 0;\n\n\
                      SET UNIQUE_CHECKS = 0;\n\n";

const BINARY_TYPES: &[&str] = &[
    "binary",
    "varbinary",
    "tinyblob",
    "blob",
    "mediumblob",
    "longblob",
];

/// Options for [`dump`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DumpOptions<'a> {
    /// Tables whose schema is dumped without any rows.
    pub skip_rows: &'a [&'a str],
}

/// How a column's values are read and written.
enum ColumnKind {
    /// Written as a hex literal with a character set introducer.
    Text { charset: String },
    /// Written as a plain hex literal.
    Binary,
    /// Numbers, dates and times, written as a quoted literal.
    Plain,
}

struct Column {
    name: String,
    kind: ColumnKind,
}

/// Writes every base table of the connected database to `writer` as a SQL script.
///
/// All rows are read from one consistent snapshot. Generated columns are not dumped.
///
/// # Errors
///
/// Returns [`BackupError::Sqlx`] if a query fails, [`BackupError::Io`] if writing fails, and
/// [`BackupError::Invalid`] if a value cannot be written as a safe literal.
pub async fn dump(
    pool: &MySqlPool,
    options: &DumpOptions<'_>,
    mut writer: impl AsyncWrite + Unpin,
) -> Result<()> {
    let mut tx = pool
        .begin_with("START TRANSACTION WITH CONSISTENT SNAPSHOT")
        .await?;
    sqlx::raw_sql("SET time_zone = '+00:00'")
        .execute(&mut *tx)
        .await?;

    writer.write_all(HEADER.as_bytes()).await?;
    for table in base_tables(&mut tx).await? {
        let quoted = quote_identifier(&table);
        let create: String = sqlx::query(AssertSqlSafe(format!("SHOW CREATE TABLE {quoted}")))
            .fetch_one(&mut *tx)
            .await?
            .try_get(1)?;
        writer
            .write_all(format!("DROP TABLE IF EXISTS {quoted};\n\n{create};\n\n").as_bytes())
            .await?;

        if !options.skip_rows.contains(&table.as_str()) {
            dump_rows(&mut tx, &table, &mut writer).await?;
        }
    }
    writer
        .write_all(b"SET FOREIGN_KEY_CHECKS = 1;\n\nSET UNIQUE_CHECKS = 1;\n")
        .await?;
    writer.flush().await?;

    tx.rollback().await?;
    Ok(())
}

/// Replaces every table of the connected database with the tables of a dump made by [`dump`].
///
/// Existing tables are dropped first, including those the dump does not contain.
///
/// # Errors
///
/// Returns [`BackupError::Sqlx`] if a statement fails and [`BackupError::Io`] if reading fails.
pub async fn restore(pool: &MySqlPool, reader: impl AsyncBufRead + Unpin) -> Result<()> {
    let mut conn = pool.acquire().await?;
    sqlx::raw_sql("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await?;
    for table in base_tables(&mut conn).await? {
        sqlx::raw_sql(AssertSqlSafe(format!(
            "DROP TABLE {}",
            quote_identifier(&table)
        )))
        .execute(&mut *conn)
        .await?;
    }

    let mut lines = reader.lines();
    let mut statement = String::new();
    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            execute_statement(&mut conn, &mut statement).await?;
            continue;
        }
        if !statement.is_empty() {
            statement.push('\n');
        }
        statement.push_str(&line);
    }
    execute_statement(&mut conn, &mut statement).await?;

    // Closed rather than returned to the pool, since the dump changes session variables.
    conn.close().await?;
    Ok(())
}

async fn execute_statement(conn: &mut MySqlConnection, statement: &mut String) -> Result<()> {
    if statement.is_empty() {
        return Ok(());
    }
    sqlx::raw_sql(AssertSqlSafe(std::mem::take(statement)))
        .execute(conn)
        .await?;
    Ok(())
}

async fn base_tables(conn: &mut MySqlConnection) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT TABLE_NAME FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE' \
         ORDER BY TABLE_NAME",
    )
    .fetch_all(conn)
    .await?)
}

async fn columns(conn: &mut MySqlConnection, table: &str) -> Result<Vec<Column>> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT COLUMN_NAME, DATA_TYPE, CHARACTER_SET_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND IS_GENERATED = 'NEVER' \
         ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(name, data_type, charset)| {
            let kind = match charset {
                Some(charset) => ColumnKind::Text { charset },
                None if BINARY_TYPES.contains(&data_type.as_str()) => ColumnKind::Binary,
                None => ColumnKind::Plain,
            };
            Column { name, kind }
        })
        .collect())
}

async fn dump_rows(
    conn: &mut MySqlConnection,
    table: &str,
    writer: &mut (impl AsyncWrite + Unpin),
) -> Result<()> {
    let columns = columns(conn, table).await?;
    let quoted_table = quote_identifier(table);
    let column_list = columns
        .iter()
        .map(|column| quote_identifier(&column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let select_list = columns
        .iter()
        .map(|column| {
            let name = quote_identifier(&column.name);
            match column.kind {
                ColumnKind::Text { .. } | ColumnKind::Binary => format!("HEX({name})"),
                ColumnKind::Plain => format!("CAST({name} AS CHAR)"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let insert_prefix = format!("INSERT INTO {quoted_table} ({column_list}) VALUES\n");

    let sql = format!("SELECT {select_list} FROM {quoted_table}");
    let mut rows = sqlx::query(AssertSqlSafe(sql)).fetch(&mut *conn);
    let mut batch = String::new();
    let mut batch_rows = 0;
    while let Some(row) = rows.try_next().await? {
        batch.push_str(if batch_rows == 0 {
            &insert_prefix
        } else {
            ",\n"
        });
        batch.push('(');
        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                batch.push_str(", ");
            }
            let value: Option<String> = row.try_get(index)?;
            push_literal(&mut batch, table, column, value)?;
        }
        batch.push(')');
        batch_rows += 1;

        if batch_rows == BATCH_ROWS {
            batch.push_str(";\n\n");
            writer.write_all(batch.as_bytes()).await?;
            batch.clear();
            batch_rows = 0;
        }
    }
    if batch_rows > 0 {
        batch.push_str(";\n\n");
        writer.write_all(batch.as_bytes()).await?;
    }
    Ok(())
}

fn push_literal(
    output: &mut String,
    table: &str,
    column: &Column,
    value: Option<String>,
) -> Result<()> {
    let Some(value) = value else {
        output.push_str("NULL");
        return Ok(());
    };
    match &column.kind {
        ColumnKind::Text { charset } => output.push_str(&format!("_{charset} X'{value}'")),
        ColumnKind::Binary => output.push_str(&format!("X'{value}'")),
        ColumnKind::Plain => {
            let safe = value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || " .:+-".contains(character));
            if !safe {
                return Err(BackupError::Invalid(format!(
                    "unsupported value in column {table}.{}",
                    column.name
                )));
            }
            output.push_str(&format!("'{value}'"));
        }
    }
    Ok(())
}

fn quote_identifier(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}
