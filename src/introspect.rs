use anyhow::Result;
use sqlx::PgPool;

use crate::model::{ColumnInfo, Schema};

/// Reads every table's columns from `information_schema.columns` for the
/// given schema (`"public"` almost always what you want) — the standard,
/// portable-across-Postgres-versions catalog view, not `pg_catalog`
/// directly, so this doesn't need version-specific system-catalog
/// knowledge.
pub async fn introspect(pool: &PgPool, schema_name: &str) -> Result<Schema> {
    let rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT table_name, column_name, data_type, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_schema = $1
        ORDER BY table_name, ordinal_position
        "#,
    )
    .bind(schema_name)
    .fetch_all(pool)
    .await?;

    let mut schema = Schema::new();
    for (table_name, column_name, data_type, is_nullable, column_default) in rows {
        schema.entry(table_name).or_default().push(ColumnInfo {
            name: column_name,
            data_type,
            is_nullable: is_nullable == "YES",
            column_default,
        });
    }
    Ok(schema)
}
