#![feature(
    decl_macro,
    macro_metavar_expr
)]


mod colour;
pub use colour::ENABLE_COLOUR;

mod level;
pub use level::{
    fatal, fatal_once,
    error, error_once,
    warn,  warn_once,
    pass,  pass_once,
    info,  info_once,
    debug, debug_once,
    trace, trace_once
};

mod filter;
pub use filter::{ LevelFilter, GLOBAL_FILTER };

mod log;
pub use log::LogEntry;

mod target;
pub use target::{ LogTarget, SingleLogTarget, LOG_TARGETS };

mod once;
pub use once::once;
