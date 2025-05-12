#[allow(unused_imports)]
use core::sync::atomic::{
    AtomicBool,
    Ordering as AtomicOrdering
};


pub macro once( $( $code:tt )* ) { {
    static SHOULD_RUN : AtomicBool = AtomicBool::new(true);
    if (SHOULD_RUN.swap(false, AtomicOrdering::Relaxed)) {
        $( $code )*
    }
} }
