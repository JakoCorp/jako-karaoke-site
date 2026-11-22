use backup::dump::{self, DumpOptions};
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::MySqlPool;

type ParentRow = (Vec<u8>, String, Option<String>, NaiveDateTime, u64, bool);
type ChildRow = (i32, Vec<u8>, Option<Vec<u8>>);

async fn parents(pool: &MySqlPool) -> Vec<ParentRow> {
    sqlx::query_as("SELECT id, name, note, created_at, total, flag FROM parents ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn children(pool: &MySqlPool) -> Vec<ChildRow> {
    sqlx::query_as("SELECT id, parent_id, data FROM children ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn table_count(pool: &MySqlPool, table: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = false)]
async fn dump_and_restore_round_trip_every_value(pool: MySqlPool) {
    sqlx::raw_sql(
        "CREATE TABLE parents (
            id BINARY(16) NOT NULL PRIMARY KEY,
            name VARCHAR(64) NOT NULL,
            note TEXT NULL,
            created_at DATETIME NOT NULL,
            total BIGINT UNSIGNED NOT NULL,
            flag TINYINT(1) NOT NULL,
            name_length INT AS (CHAR_LENGTH(name)) VIRTUAL
        ) ENGINE = InnoDB;
        CREATE TABLE children (
            id INT NOT NULL PRIMARY KEY,
            parent_id BINARY(16) NOT NULL,
            data BLOB NULL,
            CONSTRAINT FOREIGN KEY (parent_id) REFERENCES parents (id)
        ) ENGINE = InnoDB;
        CREATE TABLE tokens (token CHAR(64) NOT NULL PRIMARY KEY) ENGINE = InnoDB;
        INSERT INTO tokens VALUES ('secret');",
    )
    .execute(&pool)
    .await
    .unwrap();

    let created_at = NaiveDate::from_ymd_opt(2026, 11, 17)
        .unwrap()
        .and_hms_opt(15, 30, 0)
        .unwrap();
    let parent_rows = [
        (
            vec![0u8; 16],
            "O'Brien \\ \"quoted\"\n\nline 🎤",
            None,
            u64::MAX,
            true,
        ),
        (vec![255u8; 16], "", Some("ノート\r\n"), 0, false),
    ];
    for (id, name, note, total, flag) in &parent_rows {
        sqlx::query("INSERT INTO parents (id, name, note, created_at, total, flag) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(note)
            .bind(created_at)
            .bind(total)
            .bind(flag)
            .execute(&pool)
            .await
            .unwrap();
    }
    let all_bytes: Vec<u8> = (0..=255).collect();
    for (id, data) in [(1, Some(all_bytes)), (2, None)] {
        sqlx::query("INSERT INTO children (id, parent_id, data) VALUES (?, ?, ?)")
            .bind(id)
            .bind(vec![0u8; 16])
            .bind(data)
            .execute(&pool)
            .await
            .unwrap();
    }
    let parents_before = parents(&pool).await;
    let children_before = children(&pool).await;

    let mut output = Vec::new();
    dump::dump(
        &pool,
        &DumpOptions {
            skip_rows: &["tokens"],
        },
        &mut output,
    )
    .await
    .unwrap();

    sqlx::raw_sql("DROP TABLE children, parents; CREATE TABLE extra (id INT PRIMARY KEY);")
        .execute(&pool)
        .await
        .unwrap();

    dump::restore(&pool, output.as_slice()).await.unwrap();

    assert_eq!(parents(&pool).await, parents_before);
    assert_eq!(children(&pool).await, children_before);
    assert_eq!(table_count(&pool, "extra").await, 0);
    assert_eq!(table_count(&pool, "tokens").await, 1);
    let token_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(token_count, 0);
    let name_length: i32 = sqlx::query_scalar("SELECT name_length FROM parents WHERE total = 0")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name_length, 0);
}
