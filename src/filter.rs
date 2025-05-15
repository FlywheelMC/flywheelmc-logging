use crate::log::LogEntry;
use core::error::Error;
use core::fmt;
use core::str::FromStr;
use std::sync::RwLock;


pub static GLOBAL_FILTER : LevelFilterContainer = LevelFilterContainer(RwLock::new(LevelFilter::ALL));


pub struct LevelFilterContainer(RwLock<LevelFilter>);

impl LevelFilterContainer {

    pub fn matched_by(&self, entry : &LogEntry) -> bool {
        self.0.read().unwrap().matched_by_selfonly(entry)
    }

    pub fn set(&self, level : LevelFilter) {
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
        "trace", "debug", "info", "pass", "warn", "error", "fatal"
    ];

    pub const ALL : Self = Self { fallback : 0, modules : Vec::new() };

}

impl fmt::Display for LevelFilter {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::LEVELS[self.fallback as usize])?;
        for (module, level,) in &self.modules {
            write!(f, ",{}={}", module, Self::LEVELS[*level as usize])?;
        }
        Ok(())
    }
}

impl FromStr for LevelFilter {
    type Err = BadLevelFilter;
    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let mut fallback = 0;
        let mut modules  = Vec::new();
        let     s        = s.to_ascii_lowercase();
        for part in s.split(",") {
            if let Some((k, v,)) = part.split_once("=") {
                let k = k.trim();
                let v = v.trim();
                let Some(level) = level_from_name(v)
                    else { return Err(BadLevelFilter::UnknownLevel(v.to_string())); };
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

    /// Also checks global.
    pub fn matched_by(&self, entry : &LogEntry) -> bool {
        entry.level_index() >= self.match_min_level(entry).or_else(|| GLOBAL_FILTER.0.read().unwrap().match_min_level(entry)).unwrap_or(self.fallback)
    }

    /// Does not check global.
    pub fn matched_by_selfonly(&self, entry : &LogEntry) -> bool {
        entry.level_index() >= self.match_min_level(entry).unwrap_or(self.fallback)
    }

    fn match_min_level(&self, entry : &LogEntry) -> Option<u8> {
        let mut min_level = None;
        for (k, v,) in &self.modules {
            if let Some(right) = entry.module().strip_prefix(k) {
                if (right.is_empty() || right.starts_with("::")) {
                    min_level = Some(min_level.unwrap_or(0).max(*v));
                }
            }
        }
        min_level
    }

}


#[derive(Debug)]
pub enum BadLevelFilter {
    UnknownLevel(String)
}

impl fmt::Display for BadLevelFilter {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            Self::UnknownLevel(name) => write!(f, "unknown log level {name:?}")
        }
    }
}

/*impl Into<Box<dyn Error + Send + Sync>> for BadLevelFilter {
    fn into(self) -> Box<dyn Error + Send + Sync> {
        self.to_string().into()
    }
}*/
impl From<BadLevelFilter> for Box<dyn Error + Send + Sync> {
    fn from(value : BadLevelFilter) -> Self {
        value.to_string().into()
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
