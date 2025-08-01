/// Direct USB logging implementation - no buffering, sends directly to USB when available
/// This module provides direct USB logging that works alongside defmt/RTT logging
use defmt::*;

/// Macro to log to defmt (RTT) - USB logging will be handled by the USB task
/// Usage: usb_log!(info, "Message: {}", value);
#[macro_export]
macro_rules! usb_log {
    (info, $($arg:tt)*) => {
        {
            defmt::info!($($arg)*);
            // USB logging will be handled by the USB task when connected
        }
    };
    (warn, $($arg:tt)*) => {
        {
            defmt::warn!($($arg)*);
            // USB logging will be handled by the USB task when connected
        }
    };
    (error, $($arg:tt)*) => {
        {
            defmt::error!($($arg)*);
            // USB logging will be handled by the USB task when connected
        }
    };
    (debug, $($arg:tt)*) => {
        {
            defmt::debug!($($arg)*);
            // USB logging will be handled by the USB task when connected
        }
    };
    (trace, $($arg:tt)*) => {
        {
            defmt::trace!($($arg)*);
            // USB logging will be handled by the USB task when connected
        }
    };
}

/// Queue a formatted log message for USB transmission - DISABLED
pub fn queue_usb_log_message(_args: &core::fmt::Arguments<'_>) {
    // USB logging disabled - function kept for compatibility
}

/// Dequeue a log message for USB transmission - DISABLED
/// Returns None since USB logging is disabled
pub fn dequeue_usb_log_message() -> Option<heapless::String<256>> {
    // USB logging disabled - always return None
    None
}
