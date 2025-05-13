use crate::log::LogEntry;
use core::str::FromStr;

pub struct LevelFilter {
    fallback : u8,
    modules  : Vec<(String, u8,)>
}


impl FromStr for LevelFilter {
    type Err = BadLevelFitler;
    fn from_str(s : &str) -> Result<Self, Self::Err> {
        let mut fallback = 0;
        let mut modules  = Vec::new();
        for part in s.split(",") {
            if let Some((k, v,)) = part.split_once("=") {
                let v = v.to_ascii_lowercase();
                let Some(level) = level_from_name(&v)
                    else { return Err(BadLevelFitler::UnknownLevel(v)); };
                modules.push((k.to_string(), level));
            } else {
                let Some(level) = level_from_name(part)
                    else { return Err(BadLevelFitler::UnknownLevel(part.to_string())); };
                fallback = fallback.max(level);
            }
        }
        Ok(Self {
            fallback
            ,modules
        })
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


pub enum BadLevelFitler {
    UnknownLevel(String)
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
