use crate::error::SqlError;
use chrono::{DateTime, Utc};
use datamodel::FieldArity;
use prisma_models::{PrismaValue, Record, TypeIdentifier};
use quaint::{
    ast::{DatabaseValue, ParameterizedValue},
    connector::ResultRow,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{borrow::Borrow, io};
use uuid::Uuid;

/// An allocated representation of a `Row` returned from the database.
#[derive(Debug, Clone, Default)]
pub struct SqlRow {
    pub values: Vec<PrismaValue>,
}

impl From<SqlRow> for Record {
    fn from(row: SqlRow) -> Record {
        Record::new(row.values)
    }
}

pub trait ToSqlRow {
    /// Conversion from a database specific row to an allocated `SqlRow`. To
    /// help deciding the right types, the provided `TypeIdentifier`s should map
    /// to the returned columns in the right order.
    fn to_sql_row<'b>(self, idents: &[(TypeIdentifier, FieldArity)]) -> crate::Result<SqlRow>;
}

impl ToSqlRow for ResultRow {
    fn to_sql_row<'b>(self, idents: &[(TypeIdentifier, FieldArity)]) -> crate::Result<SqlRow> {
        let mut row = SqlRow::default();
        let row_width = idents.len();
        for (i, p_value) in self.into_iter().enumerate().take(row_width) {
            let pv = match &idents[i] {
                (type_identifier, FieldArity::List) => match p_value {
                    ParameterizedValue::Array(l) => l
                        .into_iter()
                        .map(|p_value| row_value_to_prisma_value(p_value, &type_identifier))
                        .collect::<crate::Result<Vec<_>>>()
                        .map(|vec| PrismaValue::List(vec)),

                    ParameterizedValue::Null => Ok(PrismaValue::List(Vec::new())),
                    _ => {
                        let error = io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("List field did not return an Array from database. Type identifier was {:?}. Value was {:?}.", &type_identifier, &p_value),
                        );
                        return Err(SqlError::ConversionError(error.into()));
                    }
                },
                (type_identifier, _) => row_value_to_prisma_value(p_value, &type_identifier),
            }?;

            row.values.push(pv);
        }

        Ok(row)
    }
}

pub fn row_value_to_prisma_value(
    p_value: ParameterizedValue,
    type_identifier: &TypeIdentifier,
) -> Result<PrismaValue, SqlError> {
    Ok(match type_identifier {
        TypeIdentifier::Boolean => match p_value {
            //                    ParameterizedValue::Array(vec) => PrismaValue::Boolean(b),
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::Integer(i) => PrismaValue::Boolean(i != 0),
            ParameterizedValue::Boolean(b) => PrismaValue::Boolean(b),
            _ => {
                let error = io::Error::new(io::ErrorKind::InvalidData, "Bool value not stored as bool or int");
                return Err(SqlError::ConversionError(error.into()));
            }
        },
        TypeIdentifier::Enum(_) => match p_value {
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::Enum(cow) => PrismaValue::Enum(cow.into_owned()),
            ParameterizedValue::Text(cow) => PrismaValue::Enum(cow.into_owned()),
            _ => {
                let error = io::Error::new(io::ErrorKind::InvalidData, "Enum value not stored as enum");
                return Err(SqlError::ConversionError(error.into()));
            }
        },

        TypeIdentifier::Json => match p_value {
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::Text(json) => PrismaValue::String(json.into()),
            ParameterizedValue::Json(json) => PrismaValue::String(json.to_string()),
            _ => {
                let error = io::Error::new(io::ErrorKind::InvalidData, "Json value not stored as text or json");
                return Err(SqlError::ConversionError(error.into()));
            }
        },
        TypeIdentifier::UUID => match p_value {
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::Text(uuid) => PrismaValue::Uuid(Uuid::parse_str(&uuid)?),
            ParameterizedValue::Uuid(uuid) => PrismaValue::Uuid(uuid),
            _ => {
                let error = io::Error::new(io::ErrorKind::InvalidData, "Uuid value not stored as text or uuid");
                return Err(SqlError::ConversionError(error.into()));
            }
        },
        TypeIdentifier::DateTime => match p_value {
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::DateTime(dt) => PrismaValue::DateTime(dt),
            ParameterizedValue::Integer(ts) => {
                let nsecs = ((ts % 1000) * 1_000_000) as u32;
                let secs = (ts / 1000) as i64;
                let naive = chrono::NaiveDateTime::from_timestamp(secs, nsecs);
                let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

                PrismaValue::DateTime(datetime)
            }
            ParameterizedValue::Text(dt_string) => {
                let dt = DateTime::parse_from_rfc3339(dt_string.borrow())
                    .or_else(|_| DateTime::parse_from_rfc2822(dt_string.borrow()))
                    .expect(&format!("Could not parse stored DateTime string: {}", dt_string));

                PrismaValue::DateTime(dt.with_timezone(&Utc))
            }
            _ => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidData,
                    "DateTime value not stored as datetime, int or text",
                );
                return Err(SqlError::ConversionError(error.into()));
            }
        },
        TypeIdentifier::Float => match p_value {
            ParameterizedValue::Null => PrismaValue::Null,
            ParameterizedValue::Real(f) => PrismaValue::Float(f),
            ParameterizedValue::Integer(i) => {
                PrismaValue::Float(Decimal::from_f64(i as f64).expect("f64 was not a Decimal."))
            }
            ParameterizedValue::Text(s) => PrismaValue::Float(s.parse().unwrap()),
            _ => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Float value not stored as float, int or text",
                );
                return Err(SqlError::ConversionError(error.into()));
            }
        },
        _ => PrismaValue::from(p_value),
    })
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SqlId {
    String(String),
    Int(usize),
    UUID(Uuid),
}

impl From<SqlId> for DatabaseValue<'static> {
    fn from(id: SqlId) -> Self {
        match id {
            SqlId::String(s) => s.into(),
            SqlId::Int(i) => (i as i64).into(),
            SqlId::UUID(u) => u.into(),
        }
    }
}

impl From<&SqlId> for DatabaseValue<'static> {
    fn from(id: &SqlId) -> Self {
        id.clone().into()
    }
}
