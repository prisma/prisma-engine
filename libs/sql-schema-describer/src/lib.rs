//! Database description.

use failure::Fail;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod mysql;
pub mod postgres;
pub mod sqlite;

/// description errors.
#[derive(Debug, Fail)]
pub enum SqlSchemaDescriberError {
    /// An unknown error occurred.
    #[fail(display = "unknown")]
    UnknownError,
}

/// The result type.
pub type SqlSchemaDescriberResult<T> = core::result::Result<T, SqlSchemaDescriberError>;

/// A database description connector.
#[async_trait::async_trait]
pub trait SqlSchemaDescriberBackend: Send + Sync + 'static {
    /// List the database's schemas.
    async fn list_databases(&self) -> SqlSchemaDescriberResult<Vec<String>>;
    /// Get the databases metadata.
    async fn get_metadata(&self, schema: &str) -> SqlSchemaDescriberResult<SQLMetadata>;
    /// Describe a database schema.
    async fn describe(&self, schema: &str) -> SqlSchemaDescriberResult<SqlSchema>;
}

#[derive(Serialize, Deserialize)]
pub struct SQLMetadata {
    pub table_count: usize,
    pub size_in_bytes: usize,
}

/// The result of describing a database schema.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SqlSchema {
    /// The schema's tables.
    pub tables: Vec<Table>,
    /// The schema's enums.
    pub enums: Vec<Enum>,
    /// The schema's sequences, unique to Postgres.
    pub sequences: Vec<Sequence>,
}

impl SqlSchema {
    pub fn has_table(&self, name: &str) -> bool {
        self.get_table(name).is_some()
    }

    /// Get a table.
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|x| x.name == name)
    }

    /// Get an enum.
    pub fn get_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.iter().find(|x| x.name == name)
    }

    pub fn table(&self, name: &str) -> core::result::Result<&Table, String> {
        match self.tables.iter().find(|t| t.name == name) {
            Some(t) => Ok(t),
            None => Err(format!("Table {} not found", name)),
        }
    }

    pub fn table_bang(&self, name: &str) -> &Table {
        self.table(&name).unwrap()
    }

    /// Get a sequence.
    pub fn get_sequence(&self, name: &str) -> Option<&Sequence> {
        self.sequences.iter().find(|x| x.name == name)
    }

    pub fn empty() -> SqlSchema {
        SqlSchema {
            tables: Vec::new(),
            enums: Vec::new(),
            sequences: Vec::new(),
        }
    }
}

/// A table found in a schema.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// The table's name.
    pub name: String,
    /// The table's columns.
    pub columns: Vec<Column>,
    /// The table's indices.
    pub indices: Vec<Index>,
    /// The table's primary key, if there is one.
    pub primary_key: Option<PrimaryKey>,
    /// The table's foreign keys.
    pub foreign_keys: Vec<ForeignKey>,
}

impl Table {
    pub fn column_bang(&self, name: &str) -> &Column {
        self.column(name)
            .expect(&format!("Column {} not found in Table {}", name, self.name))
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name == name)
    }

    pub fn has_column(&self, name: &str) -> bool {
        self.column(name).is_some()
    }

    pub fn is_part_of_foreign_key(&self, column: &str) -> bool {
        self.foreign_key_for_column(column).is_some()
    }

    pub fn foreign_key_for_column(&self, column: &str) -> Option<&ForeignKey> {
        self.foreign_keys
            .iter()
            .find(|fk| fk.columns.contains(&column.to_string()))
    }

    pub fn is_part_of_primary_key(&self, column: &str) -> bool {
        match &self.primary_key {
            Some(pk) => pk.columns.contains(&column.to_string()),
            None => false,
        }
    }

    pub fn primary_key_columns(&self) -> Vec<String> {
        match &self.primary_key {
            Some(pk) => pk.columns.clone(),
            None => Vec::new(),
        }
    }

    pub fn is_column_unique(&self, column_name: &str) -> bool {
        self.indices.iter().any(|index| {
            index.tpe == IndexType::Unique
                && index.columns.len() == 1
                && index.columns.contains(&column_name.to_owned())
        })
    }
}
/// The type of an index.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndexType {
    /// Unique type.
    Unique,
    /// Normal type.
    Normal,
}

impl IndexType {
    pub fn is_unique(&self) -> bool {
        match self {
            IndexType::Unique => true,
            _ => false,
        }
    }
}

/// An index of a table.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Index columns.
    pub columns: Vec<String>,
    /// Type of index.
    pub tpe: IndexType,
}

impl Index {
    pub fn is_unique(&self) -> bool {
        self.tpe == IndexType::Unique
    }
}

/// The primary key of a table.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryKey {
    /// Columns.
    pub columns: Vec<String>,
    /// The sequence optionally seeding this primary key.
    pub sequence: Option<Sequence>,
}

impl PrimaryKey {
    pub fn is_single_primary_key(&self, column: &String) -> bool {
        self.columns.len() == 1 && self.columns.contains(column)
    }
}

/// A column of a table.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column type.
    pub tpe: ColumnType,
    /// Column default.
    pub default: Option<DefaultValue>,
    /// Is the column auto-incrementing?
    pub auto_increment: bool,
}

impl Column {
    pub fn is_required(&self) -> bool {
        self.tpe.arity == ColumnArity::Required
    }
}

/// The type of a column.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnType {
    /// The raw SQL type.
    pub raw: String,
    /// The family of the raw type.
    pub family: ColumnTypeFamily,
    /// The arity of the column.
    pub arity: ColumnArity,
}

impl ColumnType {
    pub fn pure(family: ColumnTypeFamily, arity: ColumnArity) -> ColumnType {
        ColumnType {
            raw: "".to_string(),
            family,
            arity,
        }
    }
}

/// Enumeration of column type families.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
// TODO: this name feels weird.
pub enum ColumnTypeFamily {
    /// Integer types.
    Int,
    /// Floating point types.
    Float,
    /// Boolean types.
    Boolean,
    /// String types.
    String,
    /// DateTime types.
    DateTime,
    /// Binary types.
    Binary,
    /// JSON types.
    Json,
    /// UUID types.
    Uuid,
    /// Geometric types.
    Geometric,
    /// Log sequence number types.
    LogSequenceNumber,
    /// Text search types.
    TextSearch,
    /// Transaction ID types.
    TransactionId,
    ///Enum
    Enum(String),
    /// Unsupported
    Unsupported(String),
}

impl fmt::Display for ColumnTypeFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::String => "string".to_string(),
            Self::DateTime => "dateTime".to_string(),
            Self::Binary => "binary".to_string(),
            Self::Json => "json".to_string(),
            Self::Uuid => "uuid".to_string(),
            Self::Geometric => "geometric".to_string(),
            Self::LogSequenceNumber => "logSequenceNumber".to_string(),
            Self::TextSearch => "textSearch".to_string(),
            Self::TransactionId => "transactionId".to_string(),
            Self::Enum(x) => format!("Enum({})", &x),
            Self::Unsupported(x) => x.to_string(),
        };
        write!(f, "{}", str)
    }
}

/// A column's arity.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ColumnArity {
    /// Required column.
    Required,
    /// Nullable column.
    Nullable,
    /// List type column.
    List,
}

impl ColumnArity {
    pub fn is_required(&self) -> bool {
        match self {
            ColumnArity::Required => true,
            _ => false,
        }
    }
}

/// Foreign key action types (for ON DELETE|ON UPDATE).
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForeignKeyAction {
    /// Produce an error indicating that the deletion or update would create a foreign key
    /// constraint violation. If the constraint is deferred, this error will be produced at
    /// constraint check time if there still exist any referencing rows. This is the default action.
    NoAction,
    /// Produce an error indicating that the deletion or update would create a foreign key
    /// constraint violation. This is the same as NO ACTION except that the check is not deferrable.
    Restrict,
    /// Delete any rows referencing the deleted row, or update the values of the referencing
    /// column(s) to the new values of the referenced columns, respectively.
    Cascade,
    /// Set the referencing column(s) to null.
    SetNull,
    /// Set the referencing column(s) to their default values. (There must be a row in the
    /// referenced table matching the default values, if they are not null, or the operation
    /// will fail).
    SetDefault,
}

/// A foreign key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKey {
    /// The database name of the foreign key constraint, when available.
    pub constraint_name: Option<String>,
    /// Column names.
    pub columns: Vec<String>,
    /// Referenced table.
    pub referenced_table: String,
    /// Referenced columns.
    pub referenced_columns: Vec<String>,
    /// Action on deletion.
    pub on_delete_action: ForeignKeyAction,
}

impl PartialEq for ForeignKey {
    fn eq(&self, other: &Self) -> bool {
        self.columns == other.columns
            && self.referenced_table == other.referenced_table
            && self.referenced_columns == other.referenced_columns
    }
}

/// A SQL enum.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    /// Enum name.
    pub name: String,
    /// Possible enum values.
    pub values: Vec<String>,
}

/// A SQL sequence.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    /// Sequence name.
    pub name: String,
    /// Sequence initial value.
    pub initial_value: u32,
    /// Sequence allocation size.
    pub allocation_size: u32,
}

/// A DefaultValue
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum DefaultValue {
    /// A constant value, parsed as String
    VALUE(String),
    /// An expression generating a current timestamp.
    NOW,
    /// An expression generating a sequence.
    SEQUENCE(String),
    /// An unrecognized Default Value
    DBGENERATED(String),
}

impl DefaultValue {
    pub fn as_value(&self) -> Option<&str> {
        match self {
            DefaultValue::VALUE(s) => Some(s.as_str()),
            DefaultValue::DBGENERATED(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"^'?(\d+)'?$").expect("compile regex"));

fn parse_int(value: &str) -> Option<i32> {
    let rslt = RE_NUM.captures(value);
    if rslt.is_none() {
        return None;
    }

    let captures = rslt.expect("get captures");
    let num_str = captures.get(1).expect("get capture").as_str();
    let num_rslt = num_str.parse::<i32>();
    match num_rslt {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    value.to_lowercase().parse().ok()
}

static RE_FLOAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^'?([^']+)'?$").expect("compile regex"));

fn parse_float(value: &str) -> Option<f32> {
    let rslt = RE_FLOAT.captures(value);
    if rslt.is_none() {
        return None;
    }

    let captures = rslt.expect("get captures");
    let num_str = captures.get(1).expect("get capture").as_str();
    let num_rslt = num_str.parse::<f32>();
    match num_rslt {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}
