use std::fmt::Arguments;
use std::io::Write;
use std::sync::OnceLock;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level { Info=1, Debug=2, Trace=3 }

pub struct Logger { pub level: Level, pub color: bool }
pub static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn set_level(lvl: Level){ let _ = LOGGER.set(Logger{ level: lvl, color: true }); }
fn enabled(lvl: Level) -> bool { LOGGER.get().map(|lg| (lg.level as u8) >= (lvl as u8)).unwrap_or(false) }
fn indent() -> &'static str { "" }

pub(crate) fn log_impl(lvl: Level, glyph: &str, color_code: &str, msg: Arguments<'_>){
    if !enabled(lvl) { return; }
    let mut line = format!("{}{} {}", indent(), glyph, msg);
    if let Some(lg) = LOGGER.get() { if lg.color { line = format!("\x1b[{}m{}\x1b[0m", color_code, line); } }
    let _ = writeln!(&mut std::io::stderr().lock(), "{}", line);
}

#[macro_export] macro_rules! log_info  { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Info,  "ℹ", "36", format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_debug { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Debug, "●", "35", format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_trace { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Trace, "›", "90", format_args!($($t)*)) }; }
#[macro_export] macro_rules! span { ($lvl:expr, $name:expr) => { $crate::rdx_stderr::log_impl($lvl, "▶", "33", format_args!("{}", $name)) }; }

#[macro_export] macro_rules! log_warn { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Info, "!", "33", format_args!($($t)*)) }; }
