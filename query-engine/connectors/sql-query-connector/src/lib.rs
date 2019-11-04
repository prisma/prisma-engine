//! # The SQL Connector interface
//!
//! The public interface to outside is split into separate traits:
//!
//! - [DatabaseReader](../query-connector/trait.DatabaseReader.html) to fetch data.
//! - [DatabaseWriter](../query-connector/trait.DatabaseWriter.html) to write
//!   data.

mod cursor_condition;
mod database;
mod error;
mod filter_conversion;
mod ordering;
mod query_builder;
mod query_ext;
mod raw_query;
mod row;

use filter_conversion::*;
use query_ext::QueryExt;
use raw_query::*;
use row::*;

pub use database::*;
pub use error::SqlError;

type Result<T> = std::result::Result<T, error::SqlError>;
