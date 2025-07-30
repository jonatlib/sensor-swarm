use crate::hw::blackpill_f401::usb::UsbManager;
/// USB logging bridge that forwards logs to USB while keeping defmt-rtt as primary
/// This module provides a logging bridge that can send logs to both RTT (via defmt) and USB
use crate::hw::traits::UsbLogger;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use defmt::*;

/// Global USB logger instance pointer
static USB_LOGGER: AtomicPtr<UsbManager> = AtomicPtr::new(ptr::null_mut());

/// Initialize the USB logging bridge with a UsbManager instance
/// This must be called after USB initialization to enable USB logging
pub fn init_usb_logging_bridge(usb_manager: &'static mut UsbManager) {
    USB_LOGGER.store(usb_manager as *mut UsbManager, Ordering::Relaxed);
    info!("USB logging bridge initialized");
}

/// Macro to log to both defmt (RTT) and USB
/// Usage: usb_log!(info, "Message: {}", value);
#[macro_export]
macro_rules! usb_log {
    (info, $($arg:tt)*) => {
        {
            defmt::info!($($arg)*);
            $crate::hw::blackpill_f401::usb_defmt_logger::queue_usb_log_message(&format_args!($($arg)*));
        }
    };
    (warn, $($arg:tt)*) => {
        {
            defmt::warn!($($arg)*);
            $crate::hw::blackpill_f401::usb_defmt_logger::queue_usb_log_message(&format_args!($($arg)*));
        }
    };
    (error, $($arg:tt)*) => {
        {
            defmt::error!($($arg)*);
            $crate::hw::blackpill_f401::usb_defmt_logger::queue_usb_log_message(&format_args!($($arg)*));
        }
    };
    (debug, $($arg:tt)*) => {
        {
            defmt::debug!($($arg)*);
            $crate::hw::blackpill_f401::usb_defmt_logger::queue_usb_log_message(&format_args!($($arg)*));
        }
    };
    (trace, $($arg:tt)*) => {
        {
            defmt::trace!($($arg)*);
            $crate::hw::blackpill_f401::usb_defmt_logger::queue_usb_log_message(&format_args!($($arg)*));
        }
    };
}

/// Queue a formatted log message for USB transmission
pub fn queue_usb_log_message(args: &core::fmt::Arguments<'_>) {
    use heapless::String;
    let mut formatted = String::<USB_LOG_MESSAGE_SIZE>::new();
    if core::fmt::write(&mut formatted, *args).is_ok() {
        queue_usb_log_str(formatted.as_str());
    }
}

/// Simple ring buffer for USB log messages
const USB_LOG_QUEUE_SIZE: usize = 16;
const USB_LOG_MESSAGE_SIZE: usize = 256;

static mut USB_LOG_QUEUE: [Option<heapless::String<USB_LOG_MESSAGE_SIZE>>; USB_LOG_QUEUE_SIZE] =
    [const { None }; USB_LOG_QUEUE_SIZE];
static mut USB_LOG_QUEUE_HEAD: usize = 0;
static mut USB_LOG_QUEUE_TAIL: usize = 0;

/// Queue a log message for USB transmission
fn queue_usb_log_str(message: &str) {
    unsafe {
        let next_head = (USB_LOG_QUEUE_HEAD + 1) % USB_LOG_QUEUE_SIZE;
        if next_head != USB_LOG_QUEUE_TAIL {
            // Queue not full, add message
            let mut log_msg = heapless::String::<USB_LOG_MESSAGE_SIZE>::new();
            if log_msg.push_str(message).is_ok() {
                USB_LOG_QUEUE[USB_LOG_QUEUE_HEAD] = Some(log_msg);
                USB_LOG_QUEUE_HEAD = next_head;
            }
        }
        // If queue is full, drop the message (RTT will still have it)
    }
}

/// Dequeue a log message for USB transmission
/// Returns None if queue is empty
pub fn dequeue_usb_log_message() -> Option<heapless::String<USB_LOG_MESSAGE_SIZE>> {
    unsafe {
        if USB_LOG_QUEUE_HEAD != USB_LOG_QUEUE_TAIL {
            let message = USB_LOG_QUEUE[USB_LOG_QUEUE_TAIL].take();
            USB_LOG_QUEUE_TAIL = (USB_LOG_QUEUE_TAIL + 1) % USB_LOG_QUEUE_SIZE;
            message
        } else {
            None
        }
    }
}

/// Process queued USB log messages
/// This should be called from the USB device task
pub async fn process_usb_log_queue(usb_manager: &mut UsbManager) {
    while let Some(message) = dequeue_usb_log_message() {
        // Send the log message over USB
        if let Err(_) = usb_manager.log(message.as_str()).await {
            // If USB logging fails, the message is already in RTT
            break;
        }
    }
}
