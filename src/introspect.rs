use anyhow::Result;
use sqlx::PgPool;

use crate::model::{ColumnInfo, ConstraintInfo, IndexInfo, Schema};

pub async fn introspect(pool: &PgPool, schema_name: &str) -> Result<Schema> {
    let mut schema = Schema::new();

    let col_rows: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
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

    for (table_name, column_name, data_type, is_nullable, column_default) in col_rows {
        schema.entry(table_name).or_default().columns.push(ColumnInfo {
            name: column_name,
            data_type,
            is_nullable: is_nullable == "YES",
            column_default,
        });
    }

    let idx_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT 
            t.relname AS table_name,
            i.relname AS index_name,
            pg_get_indexdef(i.oid) AS index_def
        FROM pg_index ix
        JOIN pg_class t ON t.oid = ix.indrelid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE n.nspname = $1
          AND NOT EXISTS (
              SELECT 1 FROM pg_constraint c WHERE c.conindid = ix.indexrelid
          )
        ORDER BY t.relname, i.relname
        "#,
    )
    .bind(schema_name)
    .fetch_all(pool)
    .await?;

    for (table_name, index_name, index_def) in idx_rows {
        schema.entry(table_name).or_default().indexes.push(IndexInfo {
            name: index_name,
            index_def,
        });
    }

    let con_rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT 
            t.relname AS table_name,
            c.conname AS constraint_name,
            pg_get_constraintdef(c.oid) AS constraint_def
        FROM pg_constraint c
        JOIN pg_class t ON c.conrelid = t.oid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE n.nspname = $1
        ORDER BY t.relname, c.conname
        "#,
    )
    .bind(schema_name)
    .fetch_all(pool)
    .await?;

    for (table_name, constraint_name, constraint_def) in con_rows {
        schema.entry(table_name).or_default().constraints.push(ConstraintInfo {
            name: constraint_name,
            constraint_def,
        });
    }

    Ok(schema)
}
