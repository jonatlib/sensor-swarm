/// Hardware-independent Terminal module
/// This module provides a Terminal struct that can work with any UsbCdc implementation
/// The Terminal handles logging, command input/output, and can be shared between tasks

use crate::usb::UsbCdc;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use heapless::String;

/// Hardware-independent Terminal struct
/// This struct wraps a UsbCdc implementation and provides higher-level terminal functionality
pub struct Terminal<T: UsbCdc> {
    usb_cdc: T,
    initialized: bool,
}

impl<T: UsbCdc> Terminal<T> {
    /// Create a new Terminal with the given UsbCdc implementation
    pub fn new(usb_cdc: T) -> Self {
        Self {
            usb_cdc,
            initialized: false,
        }
    }

    /// Initialize the terminal (basic implementation does nothing, future TUI init goes here)
    pub async fn init(&mut self) -> Result<(), &'static str> {
        // Wait for USB connection
        self.usb_cdc.wait_connection().await;
        self.initialized = true;
        Ok(())
    }

    /// Write log message to terminal (no formatting done here)
    pub async fn write_logs(&mut self, message: &str) -> Result<(), &'static str> {
        if !self.initialized || !self.usb_cdc.is_connected() {
            return Err("Terminal not initialized or USB not connected");
        }

        // Add newline for proper terminal display
        let mut log_msg = String::<512>::new();
        if core::fmt::write(&mut log_msg, format_args!("{}\r\n", message)).is_ok() {
            match self.usb_cdc.write(log_msg.as_bytes()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err("Failed to format log message")
        }
    }

    /// Write bytes directly to terminal
    pub async fn write_bytes(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.initialized || !self.usb_cdc.is_connected() {
            return Err("Terminal not initialized or USB not connected");
        }

        self.usb_cdc.write(data).await
    }

    /// Read bytes from terminal (non-blocking)
    pub async fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.initialized || !self.usb_cdc.is_connected() {
            return Err("Terminal not initialized or USB not connected");
        }

        self.usb_cdc.read(buffer).await
    }

    /// Check if terminal is connected and ready
    pub fn is_connected(&self) -> bool {
        self.initialized && self.usb_cdc.is_connected()
    }

    /// Wait for terminal connection
    pub async fn wait_connection(&mut self) {
        self.usb_cdc.wait_connection().await;
        self.initialized = true;
    }
}

/// Shareable Terminal type using Mutex for thread-safe access
pub type SharedTerminal<T> = Mutex<NoopRawMutex, Terminal<T>>;

/// Create a shared terminal that can be used across multiple tasks
pub fn create_shared_terminal<T: UsbCdc>(usb_cdc: T) -> SharedTerminal<T> {
    Mutex::new(Terminal::new(usb_cdc))
}