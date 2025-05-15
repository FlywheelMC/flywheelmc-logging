use crate::colour::ENABLE_COLOUR;
use crate::log::{ LogEntry, usize_digits };
use crate::filter::{ LevelFilter, BadLevelFilter };
use core::error::Error;
use core::fmt;
use core::str::FromStr;
use std::fs;
use std::io::{ self, Write };
use std::path::Path;
use std::sync::RwLock;


const EMPTY : &str = "";


pub static LOG_TARGETS : LogTargetsContainer = LogTargetsContainer(RwLock::new(Vec::new()));


pub struct LogTargetsContainer(RwLock<Vec<Box<dyn LogTarget>>>);

impl LogTargetsContainer {

    pub fn push<T : LogTarget>(&self, target : T) {
        self.0.write().unwrap().push(Box::new(target));
    }
    pub fn append(&self, from : &mut Vec<Box<dyn LogTarget>>) {
        self.0.write().unwrap().append(from);
    }
    pub fn extend<I : IntoIterator<Item = Box<dyn LogTarget>>>(&self, iter : I) {
        self.0.write().unwrap().extend(iter);
    }

    pub fn handle(&self, entry : &LogEntry) {
        for target in &mut*self.0.write().unwrap() {
            let _ = target.handle(entry);
        }
    }

}


pub trait LogTarget : fmt::Debug + Send + Sync + 'static {
    fn handle(&mut self, entry : &LogEntry) -> io::Result<()>;
    fn clone_target(&self) -> Box<dyn LogTarget>;
}

pub struct SingleLogTarget(pub Box<dyn LogTarget>);

impl Clone for SingleLogTarget {
    fn clone(&self) -> Self {
        Self(self.0.clone_target())
    }
}

impl fmt::Debug for SingleLogTarget {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl FromStr for SingleLogTarget {
    type Err = BadLogTarget;
    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        let (k, v,) = if let Some((k, v,)) = s.split_once(":") {
            let k = k.trim();
            let v = LevelFilter::from_str(v.trim()).map_err(BadLogTarget::BadLevelFilter)?;
            (k, v,)
        } else {
            let s = s.trim();
            (s, LevelFilter::ALL,)
        };
        Ok(SingleLogTarget(if (k == "stderr") {
            Box::new(StderrLogTarget::default(v))
        } else if (k.starts_with("./") || k.starts_with("/")) {
            Box::new(FileLogTarget::open(v, k).map_err(BadLogTarget::Io)?)
        } else {
            return Err(BadLogTarget::UnknownTarget(k.to_string()));
        }))
    }
}

#[derive(Debug)]
pub enum BadLogTarget {
    UnknownTarget(String),
    BadLevelFilter(BadLevelFilter),
    Io(io::Error)
}

impl fmt::Display for BadLogTarget {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            Self::UnknownTarget(name) => write!(f, "unknown log target {name:?}"),
            Self::BadLevelFilter(err) => write!(f, "{err}"),
            Self::Io(err)             => write!(f, "{err}")
        }
    }
}

// impl Into<Box<dyn Error + Send + Sync>> for BadLogTarget {
//     fn into(self) -> Box<dyn Error + Send + Sync> {
//         match (self) {
//             Self::UnknownTarget(name) => name.into(),
//             Self::BadLevelFilter(err) => err.into(),
//             Self::Io(error)           => Box::new(error),
//         }
//     }
// }
impl From<BadLogTarget> for Box<dyn Error + Send + Sync> {
    fn from(value : BadLogTarget) -> Self {
        match (value) {
            BadLogTarget::UnknownTarget(name) => name.into(),
            BadLogTarget::BadLevelFilter(err) => err.into(),
            BadLogTarget::Io(error)           => Box::new(error),
        }
    }
}



pub struct StderrLogTarget {
    pub level_filter : LevelFilter,
    pub colour       : bool,
    pub writer       : io::Stderr
}

impl fmt::Debug for StderrLogTarget {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stderr")
    }
}

impl StderrLogTarget {
    pub fn default(level_filter : LevelFilter) -> Self {
        Self::new(level_filter, *ENABLE_COLOUR)
    }
    pub fn new(level_filter : LevelFilter, colour : bool) -> Self {
        Self { level_filter, colour, writer : io::stderr() }
    }
}

impl LogTarget for StderrLogTarget {
    fn handle(&mut self, entry : &LogEntry) -> io::Result<()> {
        if (! self.level_filter.matched_by(entry)) { return Ok(()); }
        let message = format!(
            "{fmt0} {fmtr}{fmt2}[{fmtr}{fmt0} {fmtr}{fmt2}{name}{fmtr}{fmt0} | {timestamp} | {module: <module_padding$} {EMPTY: >line_padding$}:{fmtr}{fmt0}{line}:{col: <col_padding$} {fmtr}{fmt2}]{fmtr}{fmt0} {fmtr} {fmt1}{message}{fmtr}\n",
            name           = entry.level_padded(),
            timestamp      = entry.timestamp_local().format("%Y-%m-%d %H:%M:%S.%f"),
            module         = entry.module(),
            line           = entry.line(),
            col            = entry.col(),
            message        = entry.message(),
            fmt0           = if (self.colour) { entry.level_format()   } else { "" },
            fmt1           = if (self.colour) { entry.message_format() } else { "" },
            fmt2           = if (self.colour) { entry.bracket_format() } else { "" },
            fmtr           = if (self.colour) { "\x1b[0m" } else { "" },
            module_padding = entry.module_padding(),
            line_padding   = (entry.line_padding() - usize_digits(entry.line() as usize).get()),
            col_padding    = entry.col_padding()
        );
        self.writer.write_all(message.as_bytes())
    }
    fn clone_target(&self) -> Box<dyn LogTarget> {
        Box::new(Self::new(self.level_filter.clone(), self.colour))
    }
}


pub struct FileLogTarget {
    pub level_filter : LevelFilter,
    pub file         : fs::File
}

impl fmt::Debug for FileLogTarget {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file")
    }
}

impl FileLogTarget {
    pub fn new(level_filter : LevelFilter, file : fs::File) -> Self {
        Self { level_filter, file }
    }
    pub fn open<P : AsRef<Path>>(level_filter : LevelFilter, fpath : P) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(fpath)?;
        Ok(Self { level_filter, file })
    }
}

impl LogTarget for FileLogTarget {
    fn handle(&mut self, entry : &LogEntry) -> io::Result<()> {
        if (! self.level_filter.matched_by(entry)) { return Ok(()); }
        let message = format!(
            "[ {name} | {timestamp} | {module: <module_padding$} {EMPTY: >line_padding$}:{line}:{col: <col_padding$} ] {message}\n",
            name           = entry.level_padded(),
            timestamp      = entry.timestamp_local().format("%Y-%m-%d %H:%M:%S.%f"),
            module         = entry.module(),
            line           = entry.line(),
            col            = entry.col(),
            message        = entry.message(),
            module_padding = entry.module_padding(),
            line_padding   = (entry.line_padding() - usize_digits(entry.line() as usize).get()),
            col_padding    = entry.col_padding()
        );
        self.file.write_all(message.as_bytes())
    }
    fn clone_target(&self) -> Box<dyn LogTarget> {
        Box::new(Self::new(self.level_filter.clone(), self.file.try_clone().unwrap()))
    }
}
