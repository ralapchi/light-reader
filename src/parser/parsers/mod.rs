pub mod base;
pub mod epub;
pub mod txt;
pub mod factory;

pub use base::{BookParser, ParseResult};
pub use epub::EpubParser;
pub use txt::TxtParser;
pub use factory::ParserFactory;
