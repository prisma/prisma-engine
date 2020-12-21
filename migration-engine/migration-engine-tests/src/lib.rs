#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]

mod assertions;
mod misc_helpers;
pub mod sql;
mod test_api;

pub use assertions::*;
pub use misc_helpers::TestResult;
pub use misc_helpers::*;
pub use test_api::*;
pub use test_macros::test_each_connector;
pub use test_setup::*;
