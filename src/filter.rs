use crate::log::LogEntry;
use core::error::Error;
use core::fmt;
use core::str::FromStr;
use std::sync::RwLock;


pub static LEVEL_FILTER : LevelFilterContainer = LevelFilterContainer(RwLock::new(LevelFilter::TRACE));


pub struct LevelFilterContainer(RwLock<LevelFilter>);

impl LevelFilterContainer {

    pub fn matched_by(&self, entry : &LogEntry) -> bool {
        self.0.read().unwrap().matched_by(entry)
    }

    pub fn set(&self, level : LevelFilter) -> () {
        *self.0.write().unwrap() = level;
    }

}


#[derive(Debug, Clone)]
pub struct LevelFilter {
    fallback : u8,
    modules  : Vec<(String, u8,)>
}

impl LevelFilter {

    pub const LEVELS : [&'static str; 7] = [
        "fatal", "error", "warn", "pass", "info", "debug", "trace"
    ];

    pub const FATAL : Self = Self { fallback : 6, modules : Vec::new() };
    pub const ERROR : Self = Self { fallback : 5, modules : Vec::new() };
    pub const WARN  : Self = Self { fallback : 4, modules : Vec::new() };
    pub const PASS  : Self = Self { fallback : 3, modules : Vec::new() };
    pub const INFO  : Self = Self { fallback : 2, modules : Vec::new() };
    pub const DEBUG : Self = Self { fallback : 1, modules : Vec::new() };
    pub const TRACE : Self = Self { fallback : 0, modules : Vec::new() };

}

impl fmt::Display for LevelFilter {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::LEVELS[self.fallback as usize])?;
        for (module, level,) in &self.modules {
            write!(f, ",{}={}", module, Self::LEVELS[self.fallback as usize])?;
        }
        Ok(())
    }
}

impl FromStr for LevelFilter {
    type Err = BadLevelFilter;
    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let mut fallback = 0;
        let mut modules  = Vec::new();
        for part in s.split(",") {
            if let Some((k, v,)) = part.split_once("=") {
                let k = k.trim();
                let v = v.trim().to_ascii_lowercase();
                let Some(level) = level_from_name(&v)
                    else { return Err(BadLevelFilter::UnknownLevel(v)); };
                modules.push((k.to_string(), level));
            } else {
                let part = part.trim();
                let Some(level) = level_from_name(part)
                    else { return Err(BadLevelFilter::UnknownLevel(part.to_string())); };
                fallback = fallback.max(level);
            }
        }
        Ok(Self { fallback, modules })
    }
}

impl LevelFilter {
    pub fn matched_by(&self, entry : &LogEntry) -> bool {
        let mut min_level = None;
        for (k, v,) in &self.modules {
            if let Some(right) = entry.module().strip_prefix(k) {
                if (right.is_empty() || right.starts_with("::")) {
                    min_level = Some(min_level.unwrap_or(0).max(*v));
                }
            }
        }
        entry.level_index() >= min_level.unwrap_or(self.fallback)
    }
}


#[derive(Debug)]
pub enum BadLevelFilter {
    UnknownLevel(String)
}

impl fmt::Display for BadLevelFilter {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            Self::UnknownLevel(name) => write!(f, "unknown level {:?}", name)
        }
    }
}

impl Into<Box<dyn Error + Send + Sync>> for BadLevelFilter {
    fn into(self) -> Box<dyn Error + Send + Sync> {
        self.to_string().into()
    }
}


fn level_from_name(name : &str) -> Option<u8> {
    Some(match (name) {
        "fatal" => 6,
        "error" => 5,
        "warn"  => 4,
        "pass"  => 3,
        "info"  => 2,
        "debug" => 1,
        "trace" => 0,
        _       => { return None; }
    })
}
