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
                Level::Info => "INFO",
                Level::Warn => "WARN",
                Level::Error => "ERROR",
            }
        )
    }
}

impl From<Level> for Style {
    fn from(level: Level) -> Style {
        match level {
            Level::Trace => Style::default().weight(Weight::Light),
            Level::Info => Style::default().color(Color::Cyan),
            Level::Warn => Style::default().weight(Weight::Bold).color(Color::Yellow),
            Level::Error => Style::default().weight(Weight::Bold).color(Color::Red),
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
            let style: $crate::style::Style = $level.into();
            print!("{}", style);
            if $level <= $crate::log::Level::Warn {
                #[allow(unused_unsafe)]
                unsafe { $crate::print_unsafe!("[{}] {}", $level, format_args!($($arg)*)) }
            } else {
                $crate::print!("[{}] {}", $level, format_args!($($arg)*))
            }
            println!("{}", $crate::style::Style::default());
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
