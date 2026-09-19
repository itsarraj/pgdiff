use crate::model::{ColumnInfo, Schema};

#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    TableAdded {
        table: String,
        columns: Vec<ColumnInfo>,
    },
    TableRemoved {
        table: String,
    },
    ColumnAdded {
        table: String,
        column: ColumnInfo,
    },
    ColumnRemoved {
        table: String,
        column: String,
    },
    TypeChanged {
        table: String,
        column: String,
        old_type: String,
        new_type: String,
    },
    NullabilityChanged {
        table: String,
        column: String,
        now_nullable: bool,
    },
}

/// Compares `old` against `new`, table by table then column by column
/// within tables that exist in both — pure, so the whole comparison logic
/// is testable without ever touching a real database (`introspect.rs` is
/// the only part of this crate that does, and it's a thin wrapper that
/// just fills in this function's two arguments from live queries).
pub fn diff_schemas(old: &Schema, new: &Schema) -> Vec<Change> {
    let mut changes = Vec::new();

    for (table, old_columns) in old {
        match new.get(table) {
            None => changes.push(Change::TableRemoved {
                table: table.clone(),
            }),
            Some(new_columns) => changes.extend(diff_columns(table, old_columns, new_columns)),
        }
    }
    for (table, new_columns) in new {
        if !old.contains_key(table) {
            changes.push(Change::TableAdded {
                table: table.clone(),
                columns: new_columns.clone(),
            });
        }
    }

    changes
}

fn diff_columns(
    table: &str,
    old_columns: &[ColumnInfo],
    new_columns: &[ColumnInfo],
) -> Vec<Change> {
    let mut changes = Vec::new();

    for old_col in old_columns {
        match new_columns.iter().find(|c| c.name == old_col.name) {
            None => changes.push(Change::ColumnRemoved {
                table: table.to_string(),
                column: old_col.name.clone(),
            }),
            Some(new_col) => {
                if old_col.data_type != new_col.data_type {
                    changes.push(Change::TypeChanged {
                        table: table.to_string(),
                        column: old_col.name.clone(),
                        old_type: old_col.data_type.clone(),
                        new_type: new_col.data_type.clone(),
                    });
                }
                if old_col.is_nullable != new_col.is_nullable {
                    changes.push(Change::NullabilityChanged {
                        table: table.to_string(),
                        column: old_col.name.clone(),
                        now_nullable: new_col.is_nullable,
                    });
                }
            }
        }
    }
    for new_col in new_columns {
        if !old_columns.iter().any(|c| c.name == new_col.name) {
            changes.push(Change::ColumnAdded {
                table: table.to_string(),
                column: new_col.clone(),
            });
        }
    }

    changes
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

    fn schema(entries: Vec<(&str, Vec<ColumnInfo>)>) -> Schema {
        entries
            .into_iter()
            .map(|(t, c)| (t.to_string(), c))
            .collect()
    }

    #[test]
    fn detects_added_and_removed_tables() {
        let old = schema(vec![("gone", vec![])]);
        let new = schema(vec![("fresh", vec![col("id", "integer", false)])]);
        let changes = diff_schemas(&old, &new);
        assert!(changes.contains(&Change::TableRemoved {
            table: "gone".to_string()
        }));
        assert!(changes
            .iter()
            .any(|c| matches!(c, Change::TableAdded { table, .. } if table == "fresh")));
    }

    #[test]
    fn detects_added_and_removed_columns() {
        let old = schema(vec![(
            "users",
            vec![col("id", "integer", false), col("legacy", "text", true)],
        )]);
        let new = schema(vec![(
            "users",
            vec![col("id", "integer", false), col("email", "text", false)],
        )]);
        let changes = diff_schemas(&old, &new);
        assert!(changes.contains(&Change::ColumnRemoved {
            table: "users".to_string(),
            column: "legacy".to_string()
        }));
        assert!(changes
            .iter()
            .any(|c| matches!(c, Change::ColumnAdded { column, .. } if column.name == "email")));
    }

    #[test]
    fn detects_a_type_change() {
        let old = schema(vec![("t", vec![col("age", "integer", false)])]);
        let new = schema(vec![("t", vec![col("age", "bigint", false)])]);
        let changes = diff_schemas(&old, &new);
        assert_eq!(
            changes,
            vec![Change::TypeChanged {
                table: "t".to_string(),
                column: "age".to_string(),
                old_type: "integer".to_string(),
                new_type: "bigint".to_string()
            }]
        );
    }

    #[test]
    fn detects_a_nullability_change() {
        let old = schema(vec![("t", vec![col("email", "text", true)])]);
        let new = schema(vec![("t", vec![col("email", "text", false)])]);
        let changes = diff_schemas(&old, &new);
        assert_eq!(
            changes,
            vec![Change::NullabilityChanged {
                table: "t".to_string(),
                column: "email".to_string(),
                now_nullable: false
            }]
        );
    }

    #[test]
    fn identical_schemas_have_no_changes() {
        let s = schema(vec![("t", vec![col("id", "integer", false)])]);
        assert!(diff_schemas(&s, &s).is_empty());
    }
}
