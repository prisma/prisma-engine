mod error;
#[cfg(feature = "sql-ext")]
pub mod sql_ext;

use chrono::prelude::*;
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use serde::{ser::Serializer, Serialize};
use std::{convert::TryFrom, fmt, string::FromUtf8Error};
use uuid::Uuid;

pub use error::ConversionFailure;
pub type PrismaValueResult<T> = std::result::Result<T, ConversionFailure>;
pub type PrismaListValue = Vec<PrismaValue>;

#[cfg(feature = "sql-ext")]
pub use sql_ext::*;

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum GraphqlId {
    String(String),
    Int(usize),
    UUID(Uuid),
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum PrismaValue {
    String(String),
    #[serde(serialize_with = "serialize_decimal")]
    Float(Decimal),
    Boolean(bool),
    #[serde(serialize_with = "serialize_date")]
    DateTime(DateTime<Utc>),
    Enum(String),
    Int(i64),
    Null,
    Uuid(Uuid),
    GraphqlId(GraphqlId),
    List(PrismaListValue),
}

impl TryFrom<serde_json::Value> for PrismaValue {
    type Error = crate::error::ConversionFailure;

    fn try_from(v: serde_json::Value) -> PrismaValueResult<Self> {
        match v {
            serde_json::Value::String(s) => Ok(PrismaValue::String(s)),
            serde_json::Value::Array(v) => {
                let vals: PrismaValueResult<Vec<PrismaValue>> = v.into_iter().map(PrismaValue::try_from).collect();
                Ok(PrismaValue::List(vals?))
            }
            serde_json::Value::Null => Ok(PrismaValue::Null),
            serde_json::Value::Bool(b) => Ok(PrismaValue::Boolean(b)),
            serde_json::Value::Number(num) => {
                if num.is_i64() {
                    Ok(PrismaValue::Int(num.as_i64().unwrap()))
                } else {
                    let fl = num.as_f64().unwrap();
                    let dec = Decimal::from_f64(fl).unwrap();

                    Ok(PrismaValue::Float(dec))
                }
            }
            serde_json::Value::Object(_) => Err(ConversionFailure::new("nested JSON object", "PrismaValue")),
        }
    }
}

fn serialize_date<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S%.3fZ")).serialize(serializer)
}

fn serialize_decimal<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    decimal.to_f64().expect("Decimal is not a f64.").serialize(serializer)
}

impl PrismaValue {
    pub fn is_null(&self) -> bool {
        match self {
            PrismaValue::Null => true,
            _ => false,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            PrismaValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<PrismaListValue> {
        match self {
            PrismaValue::List(l) => Some(l),
            _ => None,
        }
    }
}

impl fmt::Display for PrismaValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrismaValue::String(x) => x.fmt(f),
            PrismaValue::Float(x) => x.fmt(f),
            PrismaValue::Boolean(x) => x.fmt(f),
            PrismaValue::DateTime(x) => x.fmt(f),
            PrismaValue::Enum(x) => x.fmt(f),
            PrismaValue::Int(x) => x.fmt(f),
            PrismaValue::Null => "null".fmt(f),
            PrismaValue::Uuid(x) => x.fmt(f),
            PrismaValue::GraphqlId(x) => match x {
                GraphqlId::String(x) => x.fmt(f),
                GraphqlId::Int(x) => x.fmt(f),
                GraphqlId::UUID(x) => x.fmt(f),
            },
            PrismaValue::List(x) => {
                let as_string = format!("{:?}", x);
                as_string.fmt(f)
            }
        }
    }
}

impl From<&str> for PrismaValue {
    fn from(s: &str) -> Self {
        PrismaValue::from(s.to_string())
    }
}

impl From<String> for PrismaValue {
    fn from(s: String) -> Self {
        PrismaValue::String(s)
    }
}

impl TryFrom<f64> for PrismaValue {
    type Error = ConversionFailure;

    fn try_from(f: f64) -> PrismaValueResult<PrismaValue> {
        Decimal::from_f64(f)
            .map(|d| PrismaValue::Float(d))
            .ok_or(ConversionFailure::new("f32", "Decimal"))
    }
}

impl TryFrom<f32> for PrismaValue {
    type Error = ConversionFailure;

    fn try_from(f: f32) -> PrismaValueResult<PrismaValue> {
        Decimal::from_f32(f)
            .map(|d| PrismaValue::Float(d))
            .ok_or(ConversionFailure::new("f64", "Decimal"))
    }
}

impl From<bool> for PrismaValue {
    fn from(b: bool) -> Self {
        PrismaValue::Boolean(b)
    }
}

impl From<i32> for PrismaValue {
    fn from(i: i32) -> Self {
        PrismaValue::Int(i64::from(i))
    }
}

impl From<i64> for PrismaValue {
    fn from(i: i64) -> Self {
        PrismaValue::Int(i)
    }
}

impl From<usize> for PrismaValue {
    fn from(u: usize) -> Self {
        PrismaValue::Int(u as i64)
    }
}

impl From<Uuid> for PrismaValue {
    fn from(s: Uuid) -> Self {
        PrismaValue::Uuid(s)
    }
}

impl From<PrismaListValue> for PrismaValue {
    fn from(s: PrismaListValue) -> Self {
        PrismaValue::List(s)
    }
}

impl From<GraphqlId> for PrismaValue {
    fn from(id: GraphqlId) -> PrismaValue {
        PrismaValue::GraphqlId(id)
    }
}

impl From<&GraphqlId> for PrismaValue {
    fn from(id: &GraphqlId) -> PrismaValue {
        PrismaValue::GraphqlId(id.clone())
    }
}

impl TryFrom<PrismaValue> for GraphqlId {
    type Error = ConversionFailure;

    fn try_from(value: PrismaValue) -> PrismaValueResult<GraphqlId> {
        match value {
            PrismaValue::GraphqlId(id) => Ok(id),
            PrismaValue::Int(i) => Ok(GraphqlId::from(i)),
            PrismaValue::String(s) => Ok(GraphqlId::from(s)),
            PrismaValue::Uuid(u) => Ok(GraphqlId::from(u)),
            _ => Err(ConversionFailure::new("PrismaValue", "GraphqlId")),
        }
    }
}

impl TryFrom<&PrismaValue> for GraphqlId {
    type Error = ConversionFailure;

    fn try_from(value: &PrismaValue) -> PrismaValueResult<GraphqlId> {
        match value {
            PrismaValue::GraphqlId(id) => Ok(id.clone()),
            PrismaValue::Int(i) => Ok(GraphqlId::from(*i)),
            PrismaValue::String(s) => Ok(GraphqlId::from(s.clone())),
            PrismaValue::Uuid(u) => Ok(GraphqlId::from(*u)),
            _ => Err(ConversionFailure::new("PrismaValue", "GraphqlId")),
        }
    }
}

impl TryFrom<PrismaValue> for i64 {
    type Error = ConversionFailure;

    fn try_from(value: PrismaValue) -> PrismaValueResult<i64> {
        match value {
            PrismaValue::Int(i) => Ok(i),
            _ => Err(ConversionFailure::new("PrismaValue", "i64")),
        }
    }
}

impl From<&str> for GraphqlId {
    fn from(s: &str) -> Self {
        GraphqlId::from(s.to_string())
    }
}

impl From<String> for GraphqlId {
    fn from(s: String) -> Self {
        GraphqlId::String(s)
    }
}

impl TryFrom<Vec<u8>> for GraphqlId {
    type Error = FromUtf8Error;

    fn try_from(v: Vec<u8>) -> Result<GraphqlId, Self::Error> {
        Ok(GraphqlId::String(String::from_utf8(v)?))
    }
}

impl From<usize> for GraphqlId {
    fn from(id: usize) -> Self {
        GraphqlId::Int(id)
    }
}

impl From<i64> for GraphqlId {
    fn from(id: i64) -> Self {
        GraphqlId::Int(id as usize)
    }
}

impl From<u64> for GraphqlId {
    fn from(id: u64) -> Self {
        GraphqlId::Int(id as usize)
    }
}

impl From<Uuid> for GraphqlId {
    fn from(uuid: Uuid) -> Self {
        GraphqlId::UUID(uuid)
    }
}
