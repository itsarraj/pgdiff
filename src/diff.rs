use crate::model::{ColumnInfo, ConstraintInfo, IndexInfo, Schema};

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
    IndexAdded {
        table: String,
        index: IndexInfo,
    },
    IndexRemoved {
        table: String,
        index_name: String,
    },
    ConstraintAdded {
        table: String,
        constraint: ConstraintInfo,
    },
    ConstraintRemoved {
        table: String,
        constraint_name: String,
    },
}

pub fn diff_schemas(old: &Schema, new: &Schema) -> Vec<Change> {
    let mut changes = Vec::new();

    for (table, old_info) in old {
        match new.get(table) {
            None => changes.push(Change::TableRemoved {
                table: table.clone(),
            }),
            Some(new_info) => {
                changes.extend(diff_columns(table, &old_info.columns, &new_info.columns));
                changes.extend(diff_indexes(table, &old_info.indexes, &new_info.indexes));
                changes.extend(diff_constraints(table, &old_info.constraints, &new_info.constraints));
            }
        }
    }
    for (table, new_info) in new {
        if !old.contains_key(table) {
            changes.push(Change::TableAdded {
                table: table.clone(),
                columns: new_info.columns.clone(),
            });
            for idx in &new_info.indexes {
                changes.push(Change::IndexAdded { table: table.clone(), index: idx.clone() });
            }
            for con in &new_info.constraints {
                changes.push(Change::ConstraintAdded { table: table.clone(), constraint: con.clone() });
            }
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

fn diff_indexes(table: &str, old_idx: &[IndexInfo], new_idx: &[IndexInfo]) -> Vec<Change> {
    let mut changes = Vec::new();
    for o in old_idx {
        match new_idx.iter().find(|n| n.name == o.name) {
            None => changes.push(Change::IndexRemoved { table: table.to_string(), index_name: o.name.clone() }),
            Some(n) => {
                if o.index_def != n.index_def {
                    changes.push(Change::IndexRemoved { table: table.to_string(), index_name: o.name.clone() });
                    changes.push(Change::IndexAdded { table: table.to_string(), index: n.clone() });
                }
            }
        }
    }
    for n in new_idx {
        if !old_idx.iter().any(|o| o.name == n.name) {
            changes.push(Change::IndexAdded { table: table.to_string(), index: n.clone() });
        }
    }
    changes
}

fn diff_constraints(table: &str, old_con: &[ConstraintInfo], new_con: &[ConstraintInfo]) -> Vec<Change> {
    let mut changes = Vec::new();
    for o in old_con {
        match new_con.iter().find(|n| n.name == o.name) {
            None => changes.push(Change::ConstraintRemoved { table: table.to_string(), constraint_name: o.name.clone() }),
            Some(n) => {
                if o.constraint_def != n.constraint_def {
                    changes.push(Change::ConstraintRemoved { table: table.to_string(), constraint_name: o.name.clone() });
                    changes.push(Change::ConstraintAdded { table: table.to_string(), constraint: n.clone() });
                }
            }
        }
    }
    for n in new_con {
        if !old_con.iter().any(|o| o.name == n.name) {
            changes.push(Change::ConstraintAdded { table: table.to_string(), constraint: n.clone() });
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TableInfo;

    fn col(name: &str, data_type: &str, nullable: bool) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: nullable,
            column_default: None,
        }
    }

    fn schema(entries: Vec<(&str, TableInfo)>) -> Schema {
        entries
            .into_iter()
            .map(|(t, c)| (t.to_string(), c))
            .collect()
    }

    fn table(columns: Vec<ColumnInfo>) -> TableInfo {
        TableInfo {
            columns,
            indexes: vec![],
            constraints: vec![],
        }
    }

    #[test]
    fn detects_added_and_removed_tables() {
        let old = schema(vec![("gone", table(vec![]))]);
        let new = schema(vec![("fresh", table(vec![col("id", "integer", false)]))]);
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
            table(vec![col("id", "integer", false), col("legacy", "text", true)]),
        )]);
        let new = schema(vec![(
            "users",
            table(vec![col("id", "integer", false), col("email", "text", false)]),
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
        let old = schema(vec![("t", table(vec![col("age", "integer", false)]))]);
        let new = schema(vec![("t", table(vec![col("age", "bigint", false)]))]);
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
    fn identical_schemas_have_no_changes() {
        let s = schema(vec![("t", table(vec![col("id", "integer", false)]))]);
        assert!(diff_schemas(&s, &s).is_empty());
    }

    #[test]
    fn detects_indexes_and_constraints() {
        let idx = IndexInfo { name: "idx_1".to_string(), index_def: "CREATE INDEX idx_1 ON t(id)".to_string() };
        let con = ConstraintInfo { name: "con_1".to_string(), constraint_def: "PRIMARY KEY (id)".to_string() };
        
        let t_old = table(vec![col("id", "integer", false)]);
        let mut t_new = table(vec![col("id", "integer", false)]);
        t_new.indexes.push(idx.clone());
        t_new.constraints.push(con.clone());

        let changes = diff_schemas(&schema(vec![("t", t_old)]), &schema(vec![("t", t_new)]));
        assert!(changes.contains(&Change::IndexAdded { table: "t".to_string(), index: idx }));
        assert!(changes.contains(&Change::ConstraintAdded { table: "t".to_string(), constraint: con }));
    }
}
