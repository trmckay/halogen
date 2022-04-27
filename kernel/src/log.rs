use core::fmt;

use crate::prelude::*;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum Level {
    Trace = 3,
    Info = 2,
    Warn = 1,
    Error = 0,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Level::Trace => "TRACE",
                Level::Info => "INFO ",
                Level::Warn => "WARN ",
                Level::Error => "ERROR",
            }
        )
    }
}

impl From<&Level> for Style {
    fn from(level: &Level) -> Style {
        match level {
            Level::Trace => Style::new().dimmed(),
            Level::Info => Style::new().cyan(),
            Level::Warn => Style::new().yellow().bold(),
            Level::Error => Style::new().red().bold(),
        }
    }
}

lazy_static! {
    pub static ref LOG_LEVEL: Mutex<Level> = Mutex::new(Level::Warn);
}

pub fn set_level(level: Level) {
    *LOG_LEVEL.lock() = level;
}

#[macro_export]
macro_rules! _log {
    ($level:expr, $($arg:tt)*) => {
        if $level <= *$crate::log::LOG_LEVEL.lock() {
            let style = Style::from(&$level);
            #[allow(unused_unsafe)]
            unsafe {
                $crate::println_unsafe!(
                    "{}",
                    format_args!(
                        "{:.04} | {:>5} | {}",
                        (
                            riscv::register::time::read() as f64 /
                            ($crate::arch::TIMER_FREQ_HZ as f64 / 1000.0)
                        ),
                        $level,
                        format_args!($($arg)*)
                    ).style(style)
                )
            }
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        _log!($crate::log::Level::Trace, $($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        _log!($crate::log::Level::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        _log!($crate::log::Level::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        _log!($crate::log::Level::Error, $($arg)*)
    };
}
