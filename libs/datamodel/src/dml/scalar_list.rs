use crate::ast;
use crate::common::FromStrAndSpan;
use crate::error::DatamodelError;

/// Represents a strategy for embedding scalar lists.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum ScalarListStrategy {
    Embedded,
    Relation,
}

impl FromStrAndSpan for ScalarListStrategy {
    fn from_str_and_span(s: &str, span: ast::Span) -> Result<Self, DatamodelError> {
        match s {
            "EMBEDDED" => Ok(ScalarListStrategy::Embedded),
            "RELATION" => Ok(ScalarListStrategy::Relation),
            _ => Err(DatamodelError::new_literal_parser_error("id strategy", s, span)),
        }
    }
}
impl ToString for ScalarListStrategy {
    fn to_string(&self) -> String {
        match self {
            ScalarListStrategy::Embedded => String::from("EMBEDDED"),
            ScalarListStrategy::Relation => String::from("RELATION"),
        }
    }
}
