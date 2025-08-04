/// Hardware-independent logging module
/// This module provides logging functionality that uses the Terminal for output
/// It maintains compatibility with defmt for RTT logging while adding Terminal support

use heapless::String;

/// Log an info message to defmt (RTT)
pub fn log_info(args: &core::fmt::Arguments<'_>) {
    defmt::info!("{}", defmt::Display2Format(args));
}

/// Log a warning message to defmt (RTT)
pub fn log_warn(args: &core::fmt::Arguments<'_>) {
    defmt::warn!("{}", defmt::Display2Format(args));
}

/// Log an error message to defmt (RTT)
pub fn log_error(args: &core::fmt::Arguments<'_>) {
    defmt::error!("{}", defmt::Display2Format(args));
}

/// Log a debug message to defmt (RTT)
pub fn log_debug(args: &core::fmt::Arguments<'_>) {
    defmt::debug!("{}", defmt::Display2Format(args));
}

/// Log a trace message to defmt (RTT)
pub fn log_trace(args: &core::fmt::Arguments<'_>) {
    defmt::trace!("{}", defmt::Display2Format(args));
}

/// Macro for hardware-independent logging that works with Terminal
/// Usage: terminal_log!(info, "Message: {}", value);
#[macro_export]
macro_rules! terminal_log {
    (info, $($arg:tt)*) => {
        {
            // For now, just use defmt directly since async macros are complex
            // In a real implementation, you'd want to spawn a task or use a different approach
            defmt::info!($($arg)*);
            // TODO: Add terminal logging when we have proper async context
        }
    };
    (warn, $($arg:tt)*) => {
        {
            defmt::warn!($($arg)*);
            // TODO: Add terminal logging when we have proper async context
        }
    };
    (error, $($arg:tt)*) => {
        {
            defmt::error!($($arg)*);
            // TODO: Add terminal logging when we have proper async context
        }
    };
    (debug, $($arg:tt)*) => {
        {
            defmt::debug!($($arg)*);
            // TODO: Add terminal logging when we have proper async context
        }
    };
    (trace, $($arg:tt)*) => {
        {
            defmt::trace!($($arg)*);
            // TODO: Add terminal logging when we have proper async context
        }
    };
}