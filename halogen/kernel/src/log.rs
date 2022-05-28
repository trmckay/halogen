use owo_colors::Style;

pub use crate::{error, info, log, quiet, trace, warn};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum Level {
    Trace = 3,
    Info = 2,
    Warn = 1,
    Error = 0,
}

impl core::fmt::Display for Level {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

static mut LOG_LEVEL: Level = Level::Warn;

pub fn set_level(level: Level) {
    unsafe {
        LOG_LEVEL = level;
    }
}

pub fn get_level() -> Level {
    unsafe { LOG_LEVEL }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            use owo_colors::{OwoColorize, Style};

            if $level <= $crate::log::get_level() {
                let style = Style::from(&$level);
                $crate::fwprintln!(
                    "{}",
                    format_args!(
                        "{:.04} | {:>5} | {}",
                        riscv::register::time::read() as f64 /
                            ($crate::arch::TIMER_FREQ_HZ as f64 / 1000.0),
                        $level,
                        format_args!($($arg)*)
                    ).style(style)
                );
            }
        }
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        log!($crate::log::Level::Trace, $($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log!($crate::log::Level::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        log!($crate::log::Level::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        log!($crate::log::Level::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! quiet {
    { $($stmt:stmt)+ } => {
        #[allow(redundant_semicolon)]
        {
            let level = $crate::log::get_level();
            $crate::log::set_level($crate::log::Level::Error);

            $($stmt)*

            $crate::log::set_level(level);
        }
    };
}
