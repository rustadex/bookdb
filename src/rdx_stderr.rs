use std::fmt::Arguments;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy)]
pub enum Level { Info=1, Debug=2, Trace=3 }

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(1);

pub fn set_level(lvl: Level) { LOG_LEVEL.store(lvl as usize, Ordering::Relaxed); }
fn enabled(lvl: Level) -> bool { (LOG_LEVEL.load(Ordering::Relaxed)) >= (lvl as usize) }

pub(crate) fn log_impl(lvl: Level, glyph: &str, color: &str, msg: Arguments<'_>) {
    if enabled(lvl) { eprintln!("\x1b[{}m{} {}\x1b[0m", color, glyph, msg); }
}

#[macro_export] macro_rules! log_info  { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Info,  "ℹ", "36", format_args!($($t)*)); } }
#[macro_export] macro_rules! log_debug { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Debug, "⚙", "35", format_args!($($t)*)); } }
#[macro_export] macro_rules! log_trace { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Trace, "·", "90", format_args!($($t)*)); } }
#[macro_export] macro_rules! log_warn  { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Info,  "⚠", "33", format_args!($($t)*)); } }
#[macro_export] macro_rules! span     { ($level:expr, $($t:tt)*) => { $crate::rdx_stderr::log_impl($level, "⟿", "90", format_args!($($t)*)); } }
