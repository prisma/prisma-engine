use super::*;
use crate::query_document::{ParsedArgument, ParsedInputMap, ParsedInputValue};
use connector::{filter::RecordFinder, QueryArguments};
use prisma_models::{ModelRef, PrismaValue};
use std::convert::TryInto;

/// Extracts a RecordFinder from the given parsed input.
/// Expects that, assuming an extraction is possible, arguments are structurally valid, meaning that
/// previous validation guarantees that unwraps are safe. Panics otherwise.
///
/// Expects the parsed value to be transformable into a ParsedInputMap.
///
/// Errors occur if the arguments are structurally correct, but it's semantically impossible
/// to extract a record finder, e.g. if too many fields are given.
pub fn extract_record_finder(
    parsed_value: ParsedInputValue,
    model: &ModelRef,
) -> QueryGraphBuilderResult<RecordFinder> {
    let values: ParsedInputMap = parsed_value.try_into()?;
    if values.len() != 1 {
        Err(QueryGraphBuilderError::InputError(format!(
            "Expected exactly one key-value pair for 'where' argument (or implicit nested selector) for {}, got: '{}'",
            &model.name,
            values.iter().map(|v| v.0.as_str()).collect::<Vec<&str>>().join(", ")
        )))
    } else {
        let field_selector: (String, ParsedInputValue) = values.into_iter().next().unwrap();
        let model_field = model.fields().find_from_scalar(&field_selector.0).unwrap();

        match field_selector.1.try_into()? {
            PrismaValue::Null => Err(QueryGraphBuilderError::InputError(format!(
                "You provided a null value for the where clause (or implicit nested selector) on {}. Please provide a non null value.",
                &model.name
            ))),
            x => Ok(RecordFinder {
                field: model_field,
                value: x,
            }),
        }
    }
}

/// Expects the caller to know that it is structurally guaranteed that query arguments can be extracted,
/// e.g. that the query schema guarantees that required fields are present.
/// Errors occur if conversions fail unexpectedly.
pub fn extract_query_args(arguments: Vec<ParsedArgument>, model: &ModelRef) -> QueryGraphBuilderResult<QueryArguments> {
    arguments
        .into_iter()
        .fold(Ok(QueryArguments::default()), |result, arg| {
            if let Ok(res) = result {
                match arg.name.as_str() {
                    "skip" => Ok(QueryArguments {
                        skip: arg.value.try_into()?,
                        ..res
                    }),

                    "first" => Ok(QueryArguments {
                        first: arg.value.try_into()?,
                        ..res
                    }),

                    "last" => Ok(QueryArguments {
                        last: arg.value.try_into()?,
                        ..res
                    }),

                    "after" => Ok(QueryArguments {
                        after: arg.value.try_into()?,
                        ..res
                    }),

                    "before" => Ok(QueryArguments {
                        before: arg.value.try_into()?,
                        ..res
                    }),

                    "orderBy" => Ok(QueryArguments {
                        order_by: arg.value.try_into()?,
                        ..res
                    }),

                    "where" => {
                        let val: Option<ParsedInputMap> = arg.value.try_into()?;
                        match val {
                            Some(m) => {
                                let filter = Some(extract_filter(m, model)?);
                                Ok(QueryArguments { filter, ..res })
                            }
                            None => Ok(res),
                        }
                    }

                    _ => Ok(res),
                }
            } else {
                result
            }
        })
}
