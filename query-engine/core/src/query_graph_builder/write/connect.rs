use crate::{
    query_ast::*,
    query_graph::{Node, NodeRef, ParentIdsFn, QueryGraph, QueryGraphDependency},
    QueryGraphBuilderError, QueryGraphBuilderResult,
};
use prisma_models::RelationFieldRef;
use std::{convert::TryInto, sync::Arc};

// Creates a Connect node in the graph and creates edges to parent and child.
// By default, the connect node assumes that both the parent and the child node results
// are convertible to IDs, as the edges perform a transformation on the connect node to
// inject the required IDs after the parents executed.
//
// Optionally, `ParentIdsFn`s can be provided to override the default edges for parent and child, respectively.
pub fn connect_records_node(
    graph: &mut QueryGraph,
    parent: &NodeRef,
    child: &NodeRef,
    relation_field: &RelationFieldRef,
    parent_fn: Option<ParentIdsFn>,
    child_fn: Option<ParentIdsFn>,
) -> QueryGraphBuilderResult<NodeRef> {
    let connect = WriteQuery::ConnectRecords(ConnectRecords {
        parent: None,
        child: None,
        relation_field: Arc::clone(relation_field),
    });

    let connect_node = graph.create_node(Query::Write(connect));

    // Edge from parent to connect.
    graph.create_edge(
        parent,
        &connect_node,
        QueryGraphDependency::ParentIds(parent_fn.unwrap_or_else(|| {
            Box::new(|mut child_node, mut parent_ids| {
                let len = parent_ids.len();
                if len == 0 {
                    Err(QueryGraphBuilderError::AssertionError(format!(
                        "Required exactly one parent ID to be present for connect query, found none."
                    )))
                } else if len > 1 {
                    Err(QueryGraphBuilderError::AssertionError(format!(
                        "Required exactly one parent ID to be present for connect query, found {}.",
                        len
                    )))
                } else {
                    if let Node::Query(Query::Write(WriteQuery::ConnectRecords(ref mut c))) = child_node {
                        let parent_id = parent_ids.pop().unwrap();
                        c.parent = Some(parent_id.try_into()?);
                    }

                    Ok(child_node)
                }
            })
        })),
    )?;

    // Edge from child to connect.
    graph.create_edge(
        &child,
        &connect_node,
        QueryGraphDependency::ParentIds(child_fn.unwrap_or_else(|| {
            Box::new(|mut child_node, mut parent_ids| {
                let len = parent_ids.len();
                if len == 0 {
                    Err(QueryGraphBuilderError::AssertionError(format!(
                        "Required exactly one child ID to be present for connect query, found none."
                    )))
                } else if len > 1 {
                    Err(QueryGraphBuilderError::AssertionError(format!(
                        "Required exactly one child ID to be present for connect query, found {}.",
                        len
                    )))
                } else {
                    if let Node::Query(Query::Write(WriteQuery::ConnectRecords(ref mut c))) = child_node {
                        let child_id = parent_ids.pop().unwrap();
                        c.child = Some(child_id.try_into()?);
                    }

                    Ok(child_node)
                }
            })
        })),
    )?;

    Ok(connect_node)
}
