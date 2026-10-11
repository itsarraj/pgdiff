use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexInfo {
    pub name: String,
    pub index_def: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstraintInfo {
    pub name: String,
    pub constraint_def: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableInfo {
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub constraints: Vec<ConstraintInfo>,
}

impl Default for TableInfo {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            indexes: Vec::new(),
            constraints: Vec::new(),
        }
    }
}

pub type Schema = BTreeMap<String, TableInfo>;
