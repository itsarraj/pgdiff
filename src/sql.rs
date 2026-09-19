use crate::diff::Change;
use crate::model::ColumnInfo;

/// Renders one `Change` as the SQL statement that applies it — pure
/// string formatting, no escaping/quoting beyond double-quoting
/// identifiers (table/column names), since this generates *migration
/// SQL a human reviews before running*, not SQL executed automatically
/// against untrusted input; identifier quoting here is about correctness
/// with mixed-case/reserved-word names, not defending against injection.
pub fn render(change: &Change) -> String {
    match change {
        Change::TableAdded { table, columns } => render_create_table(table, columns),
        Change::TableRemoved { table } => format!("DROP TABLE \"{table}\";"),
        Change::ColumnAdded { table, column } => {
            format!(
                "ALTER TABLE \"{table}\" ADD COLUMN {};",
                render_column_def(column)
            )
        }
        Change::ColumnRemoved { table, column } => {
            format!("ALTER TABLE \"{table}\" DROP COLUMN \"{column}\";")
        }
        Change::TypeChanged {
            table,
            column,
            new_type,
            ..
        } => {
            format!("ALTER TABLE \"{table}\" ALTER COLUMN \"{column}\" TYPE {new_type};")
        }
        Change::NullabilityChanged {
            table,
            column,
            now_nullable,
        } => {
            let action = if *now_nullable {
                "DROP NOT NULL"
            } else {
                "SET NOT NULL"
            };
            format!("ALTER TABLE \"{table}\" ALTER COLUMN \"{column}\" {action};")
        }
    }
}

fn render_column_def(column: &ColumnInfo) -> String {
    let mut def = format!("\"{}\" {}", column.name, column.data_type);
    if !column.is_nullable {
        def.push_str(" NOT NULL");
    }
    if let Some(default) = &column.column_default {
        def.push_str(&format!(" DEFAULT {default}"));
    }
    def
}

fn render_create_table(table: &str, columns: &[ColumnInfo]) -> String {
    let column_defs: Vec<String> = columns.iter().map(render_column_def).collect();
    format!(
        "CREATE TABLE \"{table}\" (\n    {}\n);",
        column_defs.join(",\n    ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str, data_type: &str, nullable: bool) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: nullable,
            column_default: None,
        }
    }

    #[test]
    fn renders_drop_table() {
        assert_eq!(
            render(&Change::TableRemoved {
                table: "old_table".to_string()
            }),
            "DROP TABLE \"old_table\";"
        );
    }

    #[test]
    fn renders_create_table_with_all_columns() {
        let sql = render(&Change::TableAdded {
            table: "users".to_string(),
            columns: vec![col("id", "integer", false), col("email", "text", true)],
        });
        assert!(sql.starts_with("CREATE TABLE \"users\" ("));
        assert!(sql.contains("\"id\" integer NOT NULL"));
        assert!(sql.contains("\"email\" text"));
        assert!(
            !sql.contains("\"email\" text NOT NULL"),
            "email is nullable, must not get NOT NULL"
        );
    }

    #[test]
    fn renders_add_column_with_not_null_and_default() {
        let column = ColumnInfo {
            name: "active".to_string(),
            data_type: "boolean".to_string(),
            is_nullable: false,
            column_default: Some("true".to_string()),
        };
        let sql = render(&Change::ColumnAdded {
            table: "users".to_string(),
            column,
        });
        assert_eq!(
            sql,
            "ALTER TABLE \"users\" ADD COLUMN \"active\" boolean NOT NULL DEFAULT true;"
        );
    }

    #[test]
    fn renders_drop_column() {
        let sql = render(&Change::ColumnRemoved {
            table: "users".to_string(),
            column: "legacy".to_string(),
        });
        assert_eq!(sql, "ALTER TABLE \"users\" DROP COLUMN \"legacy\";");
    }

    #[test]
    fn renders_type_change() {
        let sql = render(&Change::TypeChanged {
            table: "t".to_string(),
            column: "age".to_string(),
            old_type: "integer".to_string(),
            new_type: "bigint".to_string(),
        });
        assert_eq!(sql, "ALTER TABLE \"t\" ALTER COLUMN \"age\" TYPE bigint;");
    }

    #[test]
    fn renders_set_and_drop_not_null() {
        let set = render(&Change::NullabilityChanged {
            table: "t".to_string(),
            column: "c".to_string(),
            now_nullable: false,
        });
        assert_eq!(set, "ALTER TABLE \"t\" ALTER COLUMN \"c\" SET NOT NULL;");
        let drop = render(&Change::NullabilityChanged {
            table: "t".to_string(),
            column: "c".to_string(),
            now_nullable: true,
        });
        assert_eq!(drop, "ALTER TABLE \"t\" ALTER COLUMN \"c\" DROP NOT NULL;");
    }
}
