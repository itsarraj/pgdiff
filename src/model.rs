use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
}

/// table name -> its columns, in a fixed (ordinal) order.
pub type Schema = BTreeMap<String, Vec<ColumnInfo>>;
