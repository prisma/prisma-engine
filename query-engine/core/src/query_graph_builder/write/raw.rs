use super::*;
use crate::{constants::inputs::args, query_ast::*, query_graph::QueryGraph, ArgumentListLookup, ParsedField};
use prisma_value::PrismaValue;
use std::convert::TryInto;

#[tracing::instrument(skip(graph, field))]
pub fn execute_raw(graph: &mut QueryGraph, field: ParsedField) -> QueryGraphBuilderResult<()> {
    let raw_query = Query::Write(WriteQuery::ExecuteRaw(raw_query(field)?));

    graph.create_node(raw_query);
    Ok(())
}

#[tracing::instrument(skip(graph, field))]
pub fn query_raw(graph: &mut QueryGraph, field: ParsedField) -> QueryGraphBuilderResult<()> {
    let raw_query = Query::Write(WriteQuery::QueryRaw(raw_query(field)?));

    graph.create_node(raw_query);
    Ok(())
}

fn raw_query(mut field: ParsedField) -> QueryGraphBuilderResult<RawQuery> {
    let query_arg = field.arguments.lookup(args::QUERY).unwrap().value;
    let parameters_arg = field.arguments.lookup(args::PARAMETERS);

    let query_value: PrismaValue = query_arg.try_into()?;
    let parameters: Vec<PrismaValue> = match parameters_arg {
        Some(parsed) => {
            let val: PrismaValue = parsed.value.try_into()?;
            val.into_list().unwrap()
        }
        None => vec![],
    };

    Ok(RawQuery {
        query: query_value.into_string().unwrap(),
        parameters,
    })
}
