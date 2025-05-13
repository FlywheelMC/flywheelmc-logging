use crate::colour::ENABLE_COLOUR;
use crate::level::LevelInfo;
use core::num::NonZeroUsize;
use std::borrow::Cow;
use std::str::Lines;
use std::sync::LazyLock;
use std::thread::{ self, Thread };
use inventory;
use chrono::{ DateTime, Utc, Local };


const EMPTY : &'static str = "";

pub const fn usize_digits(n : usize) -> NonZeroUsize {
    unsafe { NonZeroUsize::new_unchecked(if let Some(v) = n.checked_ilog10() { (v as usize) + 1 } else { 1 }) }
}


pub struct LogEntry {
    level     : &'static LevelInfo,
    module    : &'static str,
    line      : u32,
    col       : u32,
    message   : Cow<'static, str>,
    timestamp : DateTime<Utc>,
    thread    : Thread
}
impl LogEntry {

    #[inline]
    pub fn level_index(&self) -> u8 { self.level.level }

    #[inline]
    pub fn level(&self) -> &'static str { self.level.name }
    #[inline]
    pub fn level_padding(&self) -> usize { self.level.name_padded.len() - self.level.name.len() }
    #[inline]
    pub fn level_padded(&self) -> &'static str { self.level.name_padded }

    #[inline]
    pub fn level_format(&self) -> &'static str { self.level.fmt0 }
    #[inline]
    pub fn message_format(&self) -> &'static str { self.level.fmt1 }
    #[inline]
    pub fn bracket_format(&self) -> &'static str { self.level.fmt2 }

    #[inline]
    pub fn message(&self) -> &str { &self.message }
    #[inline]
    pub fn message_lines(&self) -> Lines<'_> { self.message.lines() }

    #[inline]
    pub fn timestamp_utc(&self) -> DateTime<Utc> { self.timestamp }
    #[inline]
    pub fn timestamp_local(&self) -> DateTime<Local> { DateTime::from(self.timestamp) }

    #[inline]
    pub fn module(&self) -> &'static str { self.module }
    #[inline]
    pub fn module_padding(&self) -> usize { self.module.len() }

    #[inline]
    pub fn line(&self) -> u32 { self.line }
    #[inline]
    pub fn col(&self) -> u32 { self.col }

    #[inline]
    pub fn thread(&self) -> &Thread { &self.thread }

}


pub struct ModuleLen(pub usize);
inventory::collect!(ModuleLen);
pub static MODULE_MAX_LEN : LazyLock<usize> = LazyLock::new(|| inventory::iter::<ModuleLen>.into_iter().map(|l| l.0).max().unwrap_or(0));

pub struct LineLen(pub usize);
inventory::collect!(LineLen);
pub static LINE_MAX_LEN : LazyLock<usize> = LazyLock::new(|| inventory::iter::<LineLen>.into_iter().map(|l| l.0).max().unwrap_or(0));

pub struct ColLen(pub usize);
inventory::collect!(ColLen);
pub static COL_MAX_LEN : LazyLock<usize> = LazyLock::new(|| inventory::iter::<ColLen>.into_iter().map(|l| l.0).max().unwrap_or(0));


pub macro log {
    ( $level:expr => { $fmt:tt $(,)? } ) => { log1!( $level, module_path!(), line!(), column!(), Cow::Borrowed($fmt)) },
    ( $level:expr => { $( $fmt:tt )* } ) => { log1!( $level, module_path!(), line!(), column!(), Cow::Owned(format!( $( $fmt )* ))) }
}

pub macro log1( $level:expr, $module:expr, $line:expr, $col:expr, $message:expr ) { {

    const MODULE : &'static str = $module;
    inventory::submit! { ModuleLen(MODULE.len()) }

    const LINE : u32 = $line;
    inventory::submit! { LineLen(usize_digits(LINE as usize).get()) }

    const COL : u32 = $col;
    inventory::submit! { ColLen(usize_digits(COL as usize).get()) }

    log_inner($level, MODULE, LINE, COL, $message)

} }

pub fn log_inner(
    level   : &'static LevelInfo,
    module  : &'static str,
    line    : u32,
    col     : u32,
    message : Cow<'static, str>
) -> () {
    let entry = LogEntry {
        level,
        module,
        line,
        col,
        message,
        timestamp : Utc::now(),
        thread    : thread::current()
    };
    eprintln!(
        "{fmt0} {fmtr}{fmt2}[{fmtr}{fmt0} {fmtr}{fmt2}{}{fmtr}{fmt0} | {} | {module: <module_padding$} {EMPTY: >line_padding$}:{fmtr}{fmt0}{line}:{col: <col_padding$} {fmtr}{fmt2}]{fmtr}{fmt0} {fmtr} {fmt1}{}{fmtr}",
        level.name_padded,
        entry.timestamp_local().format("%Y-%m-%d %H:%M:%S.%f"),
        entry.message,
        fmt0           = if (*ENABLE_COLOUR) { level.fmt0 } else { "" },
        fmt1           = if (*ENABLE_COLOUR) { level.fmt1 } else { "" },
        fmt2           = if (*ENABLE_COLOUR) { level.fmt2 } else { "" },
        fmtr           = if (*ENABLE_COLOUR) { "\x1b[0m" } else { "" },
        module_padding = *MODULE_MAX_LEN,
        line_padding   = (*LINE_MAX_LEN - usize_digits(line as usize).get()),
        col_padding    = *COL_MAX_LEN
    );
}
