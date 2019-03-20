use prisma_query::ast::*;

use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use std::fmt;

/// Small wrapper type that makes differentiating UUIDs easier
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UuidString(pub String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphqlId {
    String(String),
    Int(usize),
    UUID(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrismaValue {
    String(String),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Enum(String),
    Json(String),
    Int(i32),
    Relation(usize),
    Null,
    Uuid(String),
    GraphqlId(GraphqlId),
}

impl fmt::Display for PrismaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrismaValue::String(x) => x.fmt(f),
            PrismaValue::Float(x) => x.fmt(f),
            PrismaValue::Boolean(x) => x.fmt(f),
            PrismaValue::DateTime(x) => x.fmt(f),
            PrismaValue::Enum(x) => x.fmt(f),
            PrismaValue::Json(x) => x.fmt(f),
            PrismaValue::Int(x) => x.fmt(f),
            PrismaValue::Relation(x) => x.fmt(f),
            PrismaValue::Null => "null".fmt(f),
            PrismaValue::Uuid(x) => x.fmt(f),
            PrismaValue::GraphqlId(x) => match x {
                GraphqlId::String(x) => x.fmt(f),
                GraphqlId::Int(x) => x.fmt(f),
                GraphqlId::UUID(x) => x.fmt(f),
            },
        }
    }
}

impl From<GraphqlId> for DatabaseValue {
    fn from(id: GraphqlId) -> DatabaseValue {
        match id {
            GraphqlId::String(s) => s.into(),
            GraphqlId::Int(i) => (i as i64).into(),
            GraphqlId::UUID(s) => s.into(),
        }
    }
}

impl From<&GraphqlId> for DatabaseValue {
    fn from(id: &GraphqlId) -> DatabaseValue {
        id.clone().into()
    }
}

impl From<PrismaValue> for DatabaseValue {
    fn from(pv: PrismaValue) -> DatabaseValue {
        match pv {
            PrismaValue::String(s) => s.into(),
            PrismaValue::Float(f) => (f as f64).into(),
            PrismaValue::Boolean(b) => b.into(),
            PrismaValue::DateTime(d) => d.timestamp_millis().into(),
            PrismaValue::Enum(e) => e.into(),
            PrismaValue::Json(j) => j.into(),
            PrismaValue::Int(i) => (i as i64).into(),
            PrismaValue::Relation(i) => (i as i64).into(),
            PrismaValue::Null => DatabaseValue::Parameterized(ParameterizedValue::Null),
            PrismaValue::Uuid(u) => u.into(),
            PrismaValue::GraphqlId(id) => id.into(),
        }
    }
}

impl FromSql for GraphqlId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .map(|strval| GraphqlId::String(strval.to_string()))
            .or_else(|_| value.as_i64().map(|intval| GraphqlId::Int(intval as usize)))
    }
}

impl From<String> for GraphqlId {
    fn from(s: String) -> Self {
        GraphqlId::String(s)
    }
}

impl From<usize> for GraphqlId {
    fn from(id: usize) -> Self {
        GraphqlId::Int(id)
    }
}

impl From<UuidString> for GraphqlId {
    fn from(uuid: UuidString) -> Self {
        GraphqlId::UUID(uuid.0)
    }
}
