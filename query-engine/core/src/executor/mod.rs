//! What the executor module DOES:
//! - Defining an overarching executor trait, to be used on consumers of the core crate.
//! - Defining executor implementations that combine the different core modules into a coherent
//!   string of actions to execute a given query document.
//!
//! What the executor module DOES NOT DO:
//! - Define low level execution of queries. This is considered an implementation detail of the modules used by the executors.
mod interpreting_executor;
mod pipeline;

pub use interpreting_executor::*;

use crate::{query_document::Operation, response_ir::Responses, schema::QuerySchemaRef};
use async_trait::async_trait;

#[async_trait]
pub trait QueryExecutor {
    async fn execute(&self, query_doc: Operation, query_schema: QuerySchemaRef) -> crate::Result<Responses>;

    fn primary_connector(&self) -> &'static str;
}
