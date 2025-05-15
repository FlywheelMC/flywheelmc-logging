use core::ops::Deref;
use std::env;
use std::sync::LazyLock;
use supports_color::{ self, Stream };


pub static ENABLE_COLOUR : EnableColour = EnableColour(LazyLock::new(from_args_or_env));

pub struct EnableColour(LazyLock<bool>);

impl Deref for EnableColour {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}




fn from_args_or_env() -> bool {
    let mut args = env::args();

    while let Some(arg) = args.next() {
        if (arg == "-c" || arg == "--colour" || arg == "--color") {
            return match (args.next()) {
                Some(s) => from_mode_arg(&s),
                None    => from_env_only()
            };
        } else if let Some(s) = arg.strip_prefix("-c=") {
            return from_mode_arg(s);
        } else if let Some(s) = arg.strip_prefix("--colour=") {
            return from_mode_arg(s);
        } else if let Some(s) = arg.strip_prefix("--color=") {
            return from_mode_arg(s);
        }
    }
    from_env_only()
}

fn from_mode_arg(mode : &str) -> bool {
    match (mode) {
        "always" => true,
        "never"  => false,
        _        => from_env_only(),
    }
}


fn from_env_only() -> bool {
    if (env_bool("CLICOLOR_FORCE") || env_bool("FORCE_COLOR")) {
        true
    } else if (env_maybe_bool("NO_COLOR") == Some(true)) {
        false
    } else if (env_maybe_bool("CLICOLOR") == Some(true)) {
        true
    } else {
        supports_color::on(Stream::Stderr).is_some_and(|l| l.has_basic)
    }
}


fn env_bool(name : &str) -> bool {
    env_maybe_bool(name).unwrap_or(false)
}

fn env_maybe_bool(name : &str) -> Option<bool> {
    env::var(name).ok().map(|v| v != "0" && (! v.eq_ignore_ascii_case("false")))
}
