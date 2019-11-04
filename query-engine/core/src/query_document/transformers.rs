//! Transformations for the parsed query document tree.
//! As the schema validation guarantees the presence, type conformity, etc. of incoming documents,
//! consumers of the parsed query document want to directly unwrap and access the incoming data,
//! but would need to clutter their code with tons of matches and unwraps.
//! The transformers in this file helps consumers to directly access the data in the shape they
//! assume the data has to be because of the structural guarantees of the query schema validation.
use super::*;
use chrono::prelude::*;
use prisma_models::{EnumValue, EnumValueWrapper, GraphqlId, OrderBy, PrismaValue};
use serde_json::Value;
use std::convert::TryInto;

impl TryInto<PrismaValue> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<PrismaValue> {
        match self {
            ParsedInputValue::Single(val) => Ok(val),
            ParsedInputValue::List(values) => values
                .into_iter()
                .map(|val| val.try_into())
                .collect::<QueryParserResult<Vec<PrismaValue>>>()
                .map(|vec| PrismaValue::List(Some(vec))),

            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of ParsedInputValue ({:?}) into PrismaValue failed.",
                v
            ))),
        }
    }
}

impl TryInto<ParsedInputMap> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<ParsedInputMap> {
        match self {
            ParsedInputValue::Map(val) => Ok(val),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-map ParsedInputValue ({:?}) into map failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<ParsedInputMap>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<ParsedInputMap>> {
        match self {
            ParsedInputValue::Single(PrismaValue::Null) => Ok(None),
            ParsedInputValue::Map(val) => Ok(Some(val)),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-map ParsedInputValue ({:?}) into Option map failed.",
                v
            ))),
        }
    }
}

impl TryInto<Vec<ParsedInputValue>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Vec<ParsedInputValue>> {
        match self {
            ParsedInputValue::List(vals) => Ok(vals),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-list ParsedInputValue ({:?}) into list failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<String>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<String>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::String(s) => Ok(Some(s)),
            PrismaValue::Enum(s) => Ok(Some(s.as_string())),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-String Prisma value type ({:?}) into String failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<EnumValue>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<EnumValue>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::Enum(s) => Ok(Some(s)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-Enum Prisma value type ({:?}) into enum value failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<OrderBy>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<OrderBy>> {
        let enum_val: Option<EnumValue> = self.try_into()?;

        match enum_val {
            Some(EnumValue {
                value: EnumValueWrapper::OrderBy(ob),
                ..
            }) => Ok(Some(ob)),
            None => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-order-by enum Prisma value type ({:?}) into order by enum value failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<f64>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<f64>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::Float(f) => Ok(Some(f)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-float Prisma value type ({:?}) into float failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<bool>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<bool>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::Boolean(b) => Ok(Some(b)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-bool Prisma value type ({:?}) into bool failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<DateTime<Utc>>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<DateTime<Utc>>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::DateTime(dt) => Ok(Some(dt)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-DateTime Prisma value type ({:?}) into DateTime failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<Value>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<Value>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::Json(j) => Ok(Some(j)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-JSON Prisma value type ({:?}) into JSON failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<i64>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<i64>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::Int(i) => Ok(Some(i)),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-int Prisma value type ({:?}) into int failed.",
                v
            ))),
        }
    }
}

impl TryInto<Option<GraphqlId>> for ParsedInputValue {
    type Error = QueryParserError;

    fn try_into(self) -> QueryParserResult<Option<GraphqlId>> {
        let prisma_value: PrismaValue = self.try_into()?;

        match prisma_value {
            PrismaValue::GraphqlId(id) => Ok(Some(id)),
            PrismaValue::String(s) => Ok(Some(GraphqlId::String(s))),
            PrismaValue::Int(i) => Ok(Some(GraphqlId::Int(i as usize))),
            PrismaValue::Null => Ok(None),
            v => Err(QueryParserError::AssertionError(format!(
                "Attempted conversion of non-id Prisma value type ({:?}) into id failed.",
                v
            ))),
        }
    }
}
