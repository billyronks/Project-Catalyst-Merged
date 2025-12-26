//! LumaDB Schema Introspection

use serde::{Deserialize, Serialize};

/// Schema Introspector for LumaDB
pub struct SchemaIntrospector {
    db_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub namespace: String,
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<IndexSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnType {
    Uuid,
    Text,
    Integer,
    BigInt,
    Float,
    Boolean,
    Timestamp,
    Jsonb,
    Array(Box<ColumnType>),
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

impl SchemaIntrospector {
    pub fn new(db_url: &str) -> Self {
        Self {
            db_url: db_url.to_string(),
        }
    }

    /// Introspect all namespaces
    pub async fn introspect_all(&self) -> Vec<TableSchema> {
        // TODO: Query LumaDB information_schema
        vec![]
    }

    /// Introspect a specific namespace
    pub async fn introspect_namespace(&self, namespace: &str) -> Vec<TableSchema> {
        // TODO: Query LumaDB for namespace tables
        vec![]
    }

    /// Convert LumaDB column type to GraphQL type
    pub fn to_graphql_type(col_type: &ColumnType) -> String {
        match col_type {
            ColumnType::Uuid => "ID".to_string(),
            ColumnType::Text => "String".to_string(),
            ColumnType::Integer => "Int".to_string(),
            ColumnType::BigInt => "BigInt".to_string(),
            ColumnType::Float => "Float".to_string(),
            ColumnType::Boolean => "Boolean".to_string(),
            ColumnType::Timestamp => "DateTime".to_string(),
            ColumnType::Jsonb => "JSON".to_string(),
            ColumnType::Array(inner) => format!("[{}]", Self::to_graphql_type(inner)),
            ColumnType::Unknown(_) => "String".to_string(),
        }
    }
}
