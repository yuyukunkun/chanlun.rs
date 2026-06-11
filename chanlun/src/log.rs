use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// 日志模式: 0=Off, 1=Simple (eprintln), 2=Tracing (tracing subscriber)
pub static LOG_MODE: AtomicU8 = AtomicU8::new(0);

/// 向后兼容：set_log_level 设置此标志
pub static 日志启用: AtomicBool = AtomicBool::new(false);

pub fn init_from_env() {
    if let Ok(val) = std::env::var("CHANLUN_LOG_MODE") {
        match val.to_lowercase().as_str() {
            "simple" | "on" | "debug" | "1" => {
                LOG_MODE.store(1, Ordering::Relaxed);
                日志启用.store(true, Ordering::Relaxed);
            }
            "tracing" | "2" => {
                LOG_MODE.store(2, Ordering::Relaxed);
                日志启用.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

pub fn set_log_mode(mode: u8) {
    LOG_MODE.store(mode.min(2), Ordering::Relaxed);
    日志启用.store(mode > 0, Ordering::Relaxed);
}

pub fn get_log_mode() -> u8 {
    LOG_MODE.load(Ordering::Relaxed)
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if $crate::log::日志启用.load(std::sync::atomic::Ordering::Relaxed) {
            match $crate::log::LOG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                2 => tracing::warn!($($arg)*),
                _ => eprintln!($($arg)*),
            }
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if $crate::log::日志启用.load(std::sync::atomic::Ordering::Relaxed) {
            match $crate::log::LOG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                2 => tracing::error!($($arg)*),
                _ => eprintln!($($arg)*),
            }
        }
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if $crate::log::日志启用.load(std::sync::atomic::Ordering::Relaxed) {
            match $crate::log::LOG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                2 => tracing::info!($($arg)*),
                _ => println!($($arg)*),
            }
        }
    };
}
