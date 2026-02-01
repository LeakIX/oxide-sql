//! SQL Parser
//!
//! A hand-written recursive descent parser with Pratt expression parsing.

mod error;
#[allow(clippy::module_inception)]
mod parser;
mod pratt;

pub use error::ParseError;
pub use parser::Parser;
