use crate::level::LevelInfo;
use crate::target::LOG_TARGETS;
use core::num::NonZeroUsize;
use std::borrow::Cow;
use std::str::Lines;
use std::sync::LazyLock;
use std::thread::{ self, Thread };
#[allow(clippy::single_component_path_imports)]
use inventory;
use chrono::{ DateTime, Utc, Local };


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
    pub fn module_padding(&self) -> usize { *MODULE_MAX_LEN }

    #[inline]
    pub fn line(&self) -> u32 { self.line }
    #[inline]
    pub fn line_padding(&self) -> usize { *LINE_MAX_LEN }

    #[inline]
    pub fn col(&self) -> u32 { self.col }
    #[inline]
    pub fn col_padding(&self) -> usize { *COL_MAX_LEN }

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
) {
    LOG_TARGETS.handle(&LogEntry {
        level,
        module,
        line,
        col,
        message,
        timestamp : Utc::now(),
        thread    : thread::current()
    });
}
