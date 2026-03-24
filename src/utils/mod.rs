pub mod icon_resolver;
pub mod exec_parser;
pub mod fuzzy_search;

pub use icon_resolver::{find_icon_path, resolve_icon_path};
pub use exec_parser::{parse_exec_binary, binary_exists};


