//! LumaDB Schema Introspection
//!
//! Auto-discovers database schema for Hasura-style API generation.

use brivas_lumadb::{LumaDbPool, Row};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

/// Schema introspection errors
#[derive(Debug, Error)]
pub enum IntrospectionError {
    #[error("Database error: {0}")]
    Database(#[from] brivas_lumadb::LumaDbError),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, IntrospectionError>;

/// Schema Introspector for LumaDB
pub struct SchemaIntrospector {
    pool: LumaDbPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub namespace: String,
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<IndexSchema>,
    pub row_count_estimate: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColumnType {
    Uuid,
    Text,
    Varchar(Option<i32>),
    Integer,
    BigInt,
    Serial,
    BigSerial,
    Float,
    DoublePrecision,
    Boolean,
    Timestamp,
    TimestampTz,
    Date,
    Time,
    Jsonb,
    Json,
    Array(Box<ColumnType>),
    Unknown(String),
}

impl ColumnType {
    /// Parse PostgreSQL data type string into ColumnType
    pub fn from_pg_type(type_name: &str) -> Self {
        let type_lower = type_name.to_lowercase();
        
        // Handle array types
        if type_lower.starts_with("_") || type_lower.ends_with("[]") {
            let inner = type_lower.trim_start_matches('_').trim_end_matches("[]");
            return ColumnType::Array(Box::new(Self::from_pg_type(inner)));
        }
        
        match type_lower.as_str() {
            "uuid" => ColumnType::Uuid,
            "text" => ColumnType::Text,
            "character varying" | "varchar" => ColumnType::Varchar(None),
            "integer" | "int" | "int4" => ColumnType::Integer,
            "bigint" | "int8" => ColumnType::BigInt,
            "serial" | "serial4" => ColumnType::Serial,
            "bigserial" | "serial8" => ColumnType::BigSerial,
            "real" | "float4" => ColumnType::Float,
            "double precision" | "float8" => ColumnType::DoublePrecision,
            "boolean" | "bool" => ColumnType::Boolean,
            "timestamp without time zone" | "timestamp" => ColumnType::Timestamp,
            "timestamp with time zone" | "timestamptz" => ColumnType::TimestampTz,
            "date" => ColumnType::Date,
            "time" | "time without time zone" => ColumnType::Time,
            "jsonb" => ColumnType::Jsonb,
            "json" => ColumnType::Json,
            _ => ColumnType::Unknown(type_name.to_string()),
        }
    }
    
    /// Convert to GraphQL type string
    pub fn to_graphql_type(&self) -> String {
        match self {
            ColumnType::Uuid => "ID".to_string(),
            ColumnType::Text | ColumnType::Varchar(_) => "String".to_string(),
            ColumnType::Integer | ColumnType::Serial => "Int".to_string(),
            ColumnType::BigInt | ColumnType::BigSerial => "BigInt".to_string(),
            ColumnType::Float | ColumnType::DoublePrecision => "Float".to_string(),
            ColumnType::Boolean => "Boolean".to_string(),
            ColumnType::Timestamp | ColumnType::TimestampTz => "DateTime".to_string(),
            ColumnType::Date => "Date".to_string(),
            ColumnType::Time => "Time".to_string(),
            ColumnType::Jsonb | ColumnType::Json => "JSON".to_string(),
            ColumnType::Array(inner) => format!("[{}]", inner.to_graphql_type()),
            ColumnType::Unknown(_) => "String".to_string(),
        }
    }
    
    /// Convert to REST API type string
    pub fn to_rest_type(&self) -> &'static str {
        match self {
            ColumnType::Uuid => "string",
            ColumnType::Text | ColumnType::Varchar(_) => "string",
            ColumnType::Integer | ColumnType::Serial => "integer",
            ColumnType::BigInt | ColumnType::BigSerial => "integer",
            ColumnType::Float | ColumnType::DoublePrecision => "number",
            ColumnType::Boolean => "boolean",
            ColumnType::Timestamp | ColumnType::TimestampTz | ColumnType::Date | ColumnType::Time => "string",
            ColumnType::Jsonb | ColumnType::Json => "object",
            ColumnType::Array(_) => "array",
            ColumnType::Unknown(_) => "string",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// API endpoint information generated from schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub table_name: String,
    pub graphql_query: String,
    pub graphql_mutation: String,
    pub rest_list: String,
    pub rest_get: String,
    pub rest_create: String,
    pub rest_update: String,
    pub rest_delete: String,
    pub columns: Vec<ColumnSchema>,
}

impl SchemaIntrospector {
    /// Create a new schema introspector with existing pool
    pub fn new(pool: LumaDbPool) -> Self {
        Self { pool }
    }
    
    /// Create a new schema introspector from URL
    pub async fn from_url(db_url: &str) -> Result<Self> {
        let config = brivas_lumadb::PoolConfig {
            url: db_url.to_string(),
            max_size: 4,
            min_idle: Some(1),
        };
        let pool = LumaDbPool::new(config).await?;
        Ok(Self { pool })
    }

    /// Introspect all tables in public schema
    pub async fn introspect_all(&self) -> Result<Vec<TableSchema>> {
        self.introspect_namespace("public").await
    }

    /// Introspect a specific namespace/schema
    pub async fn introspect_namespace(&self, namespace: &str) -> Result<Vec<TableSchema>> {
        info!(namespace = %namespace, "Introspecting schema");
        
        let conn = self.pool.get().await?;
        
        // Get all tables
        let table_query = r#"
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = $1 
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
        "#;
        
        let rows = conn.query(table_query, &[&namespace]).await
            .map_err(|e| IntrospectionError::Parse(e.to_string()))?;
        
        let mut schemas = Vec::with_capacity(rows.len());
        
        for row in rows {
            let table_name: String = row.get(0);
            let table_schema = self.introspect_table(namespace, &table_name).await?;
            schemas.push(table_schema);
        }
        
        info!(count = schemas.len(), "Introspection complete");
        Ok(schemas)
    }

    /// Introspect a single table
    pub async fn introspect_table(&self, namespace: &str, table_name: &str) -> Result<TableSchema> {
        debug!(table = %table_name, "Introspecting table");
        
        let conn = self.pool.get().await?;
        
        // Get columns
        let column_query = r#"
            SELECT 
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                COALESCE(
                    (SELECT true FROM information_schema.table_constraints tc
                     JOIN information_schema.key_column_usage kcu 
                       ON tc.constraint_name = kcu.constraint_name
                     WHERE tc.table_schema = c.table_schema 
                       AND tc.table_name = c.table_name
                       AND tc.constraint_type = 'PRIMARY KEY'
                       AND kcu.column_name = c.column_name
                     LIMIT 1),
                    false
                ) as is_pk
            FROM information_schema.columns c
            WHERE c.table_schema = $1 
              AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#;
        
        let column_rows = conn.query(column_query, &[&namespace, &table_name]).await
            .map_err(|e| IntrospectionError::Parse(e.to_string()))?;
        
        let mut columns = Vec::with_capacity(column_rows.len());
        let mut primary_key = Vec::new();
        
        for row in column_rows {
            let col_name: String = row.get(0);
            let data_type: String = row.get(1);
            let is_nullable: String = row.get(2);
            let default: Option<String> = row.get(3);
            let is_pk: bool = row.get(4);
            
            if is_pk {
                primary_key.push(col_name.clone());
            }
            
            columns.push(ColumnSchema {
                name: col_name,
                column_type: ColumnType::from_pg_type(&data_type),
                nullable: is_nullable == "YES",
                default,
                is_primary_key: is_pk,
            });
        }
        
        // Get indexes
        let index_query = r#"
            SELECT 
                i.relname as index_name,
                array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as columns,
                ix.indisunique as is_unique
            FROM pg_class t
            JOIN pg_index ix ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            JOIN pg_namespace n ON n.oid = t.relnamespace
            WHERE n.nspname = $1 
              AND t.relname = $2
            GROUP BY i.relname, ix.indisunique
        "#;
        
        let index_rows = conn.query(index_query, &[&namespace, &table_name]).await
            .unwrap_or_default();
        
        let indexes: Vec<IndexSchema> = index_rows.iter().map(|row| {
            IndexSchema {
                name: row.get(0),
                columns: row.get::<_, Vec<String>>(1),
                unique: row.get(2),
            }
        }).collect();
        
        // Get row count estimate
        let count_query = r#"
            SELECT reltuples::bigint 
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = $1 AND c.relname = $2
        "#;
        
        let row_count_estimate: Option<i64> = conn
            .query_opt(count_query, &[&namespace, &table_name])
            .await
            .ok()
            .flatten()
            .map(|row| row.get(0));
        
        Ok(TableSchema {
            namespace: namespace.to_string(),
            name: table_name.to_string(),
            columns,
            primary_key,
            indexes,
            row_count_estimate,
        })
    }
    
    /// Generate API endpoints from discovered schemas
    pub fn generate_api_endpoints(&self, schemas: &[TableSchema]) -> Vec<ApiEndpoint> {
        schemas.iter().map(|schema| {
            let table = &schema.name;
            
            ApiEndpoint {
                table_name: table.clone(),
                graphql_query: format!("{}(limit: Int, offset: Int, where: {}WhereInput): [{}!]!", table, table, Self::to_pascal_case(table)),
                graphql_mutation: format!("insert_{}(objects: [{}InsertInput!]!): {}MutationResponse!", table, table, Self::to_pascal_case(table)),
                rest_list: format!("/v1/rest/{}", table),
                rest_get: format!("/v1/rest/{}/{{id}}", table),
                rest_create: format!("/v1/rest/{}", table),
                rest_update: format!("/v1/rest/{}/{{id}}", table),
                rest_delete: format!("/v1/rest/{}/{{id}}", table),
                columns: schema.columns.clone(),
            }
        }).collect()
    }
    
    /// Convert snake_case to PascalCase
    fn to_pascal_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_from_pg_type() {
        assert_eq!(ColumnType::from_pg_type("uuid"), ColumnType::Uuid);
        assert_eq!(ColumnType::from_pg_type("text"), ColumnType::Text);
        assert_eq!(ColumnType::from_pg_type("integer"), ColumnType::Integer);
        assert_eq!(ColumnType::from_pg_type("bigint"), ColumnType::BigInt);
        assert_eq!(ColumnType::from_pg_type("boolean"), ColumnType::Boolean);
        assert_eq!(ColumnType::from_pg_type("jsonb"), ColumnType::Jsonb);
        assert_eq!(ColumnType::from_pg_type("timestamp without time zone"), ColumnType::Timestamp);
    }
    
    #[test]
    fn test_column_type_array() {
        match ColumnType::from_pg_type("_text") {
            ColumnType::Array(inner) => assert_eq!(*inner, ColumnType::Text),
            _ => panic!("Expected array type"),
        }
    }
    
    #[test]
    fn test_to_graphql_type() {
        assert_eq!(ColumnType::Uuid.to_graphql_type(), "ID");
        assert_eq!(ColumnType::Text.to_graphql_type(), "String");
        assert_eq!(ColumnType::Integer.to_graphql_type(), "Int");
        assert_eq!(ColumnType::Jsonb.to_graphql_type(), "JSON");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(SchemaIntrospector::to_pascal_case("user_accounts"), "UserAccounts");
        assert_eq!(SchemaIntrospector::to_pascal_case("sms_history"), "SmsHistory");
        assert_eq!(SchemaIntrospector::to_pascal_case("campaigns"), "Campaigns");
    }
}
