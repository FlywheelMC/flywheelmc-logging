#[allow(unused_imports)]
use const_str::split;


pub struct LevelInfo {
    pub(crate) level       : u8,
    pub(crate) name        : &'static str,
    pub(crate) name_padded : &'static str,
    pub(crate) fmt0        : &'static str,
    pub(crate) fmt1        : &'static str,
    pub(crate) fmt2        : &'static str
}


macro level(
    $vis:vis $ident:ident $ident_once:ident { $level:tt $name:tt, $fmt0:tt, $fmt1:tt, $fmt2:tt $(,)? }
) {

    pub macro $ident( $$( $$msg:tt )* ) {
        $crate::log::log!( &LevelInfo {
            level       : $level,
            name        : split!($name, " ")[0],
            name_padded : $name,
            fmt0        : $fmt0,
            fmt1        : $fmt1,
            fmt2        : $fmt2
        } => { $$( $$msg )* } )
    }

    pub macro $ident_once( $$( $$msg:tt )* ) {
        $crate::once!( $ident( $$( $$msg )* ) )
    }

}
level! { pub fatal fatal_once { 6 "FATAL", "\x1b[91m\x1b[107m\x1b[1m", "\x1b[97m\x1b[101m\x1b[1m", "\x1b[91m\x1b[107m\x1b[1m\x1b[5m\x1b[6m", } }
level! { pub error error_once { 5 "ERROR", "\x1b[97m\x1b[101m\x1b[1m", "\x1b[91m\x1b[1m",          "\x1b[97m\x1b[101m\x1b[1m",               } }
level! { pub warn  warn_once  { 4 "WARN ", "\x1b[30m\x1b[103m",        "\x1b[93m\x1b[1m",          "\x1b[30m\x1b[103m",                      } }
level! { pub pass  pass_once  { 3 "PASS ", "\x1b[30m\x1b[42m",         "\x1b[92m",                 "\x1b[30m\x1b[42m",                       } }
level! { pub info  info_once  { 2 "INFO ", "\x1b[30m\x1b[46m",         "\x1b[36m",                 "\x1b[30m\x1b[46m",                       } }
level! { pub debug debug_once { 1 "DEBUG", "\x1b[30m\x1b[47m",         "\x1b[37m",                 "\x1b[30m\x1b[47m",                       } }
level! { pub trace trace_once { 0 "TRACE", "\x1b[37m\x1b[100m\x1b[2m", "\x1b[97m\x1b[2m",          "\x1b[37m\x1b[100m\x1b[2m", } }
