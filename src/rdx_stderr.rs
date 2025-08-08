use std::{cell::Cell, fmt::Arguments, io::{IsTerminal, Write}, sync::OnceLock, time::Instant};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level { Off, Error, Warn, Info, Debug, Trace }

pub struct Logger { level: Level, color: bool }
static LOGGER: OnceLock<Logger> = OnceLock::new();
thread_local! { static DEPTH: Cell<usize> = Cell::new(0); }

pub fn init_from_env() {
    let lvl = if std::env::var_os("BOOKDB_TRACE").is_some() { Level::Trace }
        else if std::env::var_os("BOOKDB_DEBUG").is_some() { Level::Debug }
        else if std::env::var_os("BOOKDB_DEV").is_some()   { Level::Debug }
        else { Level::Info };
    let mut color = std::io::stderr().is_terminal();
    if std::env::var_os("NO_COLOR").is_some() || std::env::var_os("BOOKDB_COLOR")==Some("0".into()) { color=false; }
    let _ = LOGGER.set(Logger{ level:lvl, color });
}
#[inline] pub fn set_level(l: Level){ if let Some(lg)=LOGGER.get(){ let _=LOGGER.set(Logger{level:l,color:lg.color}); } }
#[inline] fn enabled(l: Level)->bool{ LOGGER.get().map(|lg| l<=lg.level).unwrap_or(false) }
#[inline] fn indent()->String{ DEPTH.with(|d| "  ".repeat(d.get())) }
fn log_impl(lvl: Level, glyph: &str, color_code: &str, msg: Arguments<'_>){
    if !enabled(lvl){return;}
    let mut line = format!("{}{} {}", indent(), glyph, msg);
    if let Some(lg)=LOGGER.get(){ if lg.color { line = format!("\x1b[{}m{}\x1b[0m", color_code, line); } }
    let _ = writeln!(&mut std::io::stderr().lock(), "{line}");
}
#[macro_export] macro_rules! log_error { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Error,"✗","31;1",format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_warn  { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Warn ,"⚠","33;1",format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_info  { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Info ,"ℹ","36;1",format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_debug { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Debug,"•","90",  format_args!($($t)*)) }; }
#[macro_export] macro_rules! log_trace { ($($t:tt)*) => { $crate::rdx_stderr::log_impl($crate::rdx_stderr::Level::Trace,"→","35",  format_args!($($t)*)) }; }

pub struct Span { lvl: Level, t0: Instant, color_code: &'static str }
impl Span {
    pub fn new(lvl: Level, name: impl Into<String>) -> Option<Self> {
        if !enabled(lvl){ return None; }
        let code = match lvl{Level::Trace=>"35",Level::Debug=>"90",Level::Info=>"36;1",Level::Warn=>"33;1",Level::Error=>"31;1",Level::Off=>"0"};
        log_impl(lvl,"▸",code,format_args!("{}", name.into()));
        DEPTH.with(|d| d.set(d.get()+1));
        Some(Self{ lvl, t0: std::time::Instant::now(), color_code: code })
    }
}
impl Drop for Span{
    fn drop(&mut self){
        DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        let ms = self.t0.elapsed().as_millis();
        log_impl(self.lvl,"◂",self.color_code,format_args!("done in {ms}ms"));
    }
}
#[macro_export] macro_rules! span { ($lvl:expr,$name:expr)=>{ let _guard = $crate::rdx_stderr::Span::new($lvl,$name); }; }
