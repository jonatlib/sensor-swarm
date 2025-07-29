/// USB communication implementation for STM32F401 Black Pill
/// Provides hardware-specific USB byte-level communication and logging
/// 
/// Note: This is a stub implementation since USB support is not available
/// in embassy-stm32 version 0.1.0 for STM32F401

use crate::hw::traits::{UsbCommunication, UsbLogger, DebugInterface};
use defmt::*;

/// USB Communication Manager for STM32F401 Black Pill
/// This is a stub implementation that doesn't provide actual USB functionality
pub struct UsbManager {
    connected: bool,
}

impl UsbManager {
    /// Create a new USB Manager instance
    pub fn new() -> Self {
        Self {
            connected: false,
        }
    }

    /// Initialize USB peripheral (stub implementation)
    pub async fn init_with_peripheral(
        &mut self,
        _usb: (),  // Placeholder since USB types are not available
        _dp: (),   // Placeholder
        _dm: (),   // Placeholder
    ) -> Result<(), &'static str> {
        warn!("USB functionality not available - using stub implementation");
        Ok(())
    }
}

impl UsbCommunication for UsbManager {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<(), &'static str> {
        warn!("USB send_bytes called but USB not available: {} bytes", data.len());
        Err("USB not available")
    }

    async fn receive_bytes(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        warn!("USB receive_bytes called but USB not available: buffer size {} bytes", buffer.len());
        Err("USB not available")
    }

    fn is_connected(&self) -> bool {
        false // Always false since USB is not available
    }
}

impl UsbLogger for UsbManager {
    async fn log(&mut self, message: &str) -> Result<(), &'static str> {
        // Fall back to defmt logging since USB is not available
        info!("USB Log: {}", message);
        Ok(())
    }

    async fn log_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result<(), &'static str> {
        use heapless::String;
        
        // Format the message into a heapless string (limited to 256 chars)
        let mut formatted = String::<256>::new();
        match core::fmt::write(&mut formatted, args) {
            Ok(_) => {
                self.log(formatted.as_str()).await
            }
            Err(_) => {
                error!("Failed to format log message");
                Err("Log message formatting failed")
            }
        }
    }
}

impl DebugInterface for UsbManager {
    /// Initialize the debug interface (stub implementation)
    fn init(&mut self) -> impl core::future::Future<Output = Result<(), &'static str>> + Send {
        async move {
            info!("Debug interface initialized (stub - USB not available)");
            Ok(())
        }
    }
}

impl Default for UsbManager {
    fn default() -> Self {
        Self::new()
    }
}