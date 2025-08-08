/// Hardware-agnostic USB communication module
/// This module provides USB CDC communication functionality that is independent of specific hardware implementations
/// The UsbManager for hardware-specific initialization remains in the hw module

// Use hardware-abstracted type aliases from hw module
use crate::hw::{CurrentUsbDriver, CurrentCdcAcmClass};

/// Trait for hardware-dependent USB CDC serial communication
/// This trait provides basic read/write operations for USB serial communication
/// Implementations should handle the actual USB CDC-ACM communication
pub trait UsbCdc {
    /// Write bytes to USB CDC
    /// Returns the number of bytes written or an error
    fn write(
        &mut self,
        data: &[u8],
    ) -> impl core::future::Future<Output = Result<usize, &'static str>>;

    /// Read bytes from USB CDC (non-blocking)
    /// Returns the number of bytes read or an error
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = Result<usize, &'static str>>;

    /// Check if USB CDC is connected and ready for communication
    fn is_connected(&self) -> bool;

    /// Wait for USB CDC connection
    fn wait_connection(&mut self) -> impl core::future::Future<Output = ()>;
}

#[cfg(feature = "blackpill-f401")]
/// Simple USB CDC wrapper that implements the UsbCdc trait
/// This struct provides basic read/write operations for USB CDC communication
pub struct UsbCdcWrapper {
    cdc_class: CurrentCdcAcmClass,
    connected: bool,
}

#[cfg(feature = "blackpill-f401")]
impl UsbCdcWrapper {
    /// Create a new USB CDC wrapper with the given CDC class
    pub fn new(cdc_class: CurrentCdcAcmClass) -> Self {
        Self {
            cdc_class,
            connected: false,
        }
    }
}

#[cfg(feature = "pipico")]
/// Simple USB CDC wrapper placeholder for PiPico
/// This struct provides placeholder USB CDC functionality for RP2040
pub struct UsbCdcWrapper {
    connected: bool,
}

#[cfg(feature = "pipico")]
impl UsbCdcWrapper {
    /// Create a new USB CDC wrapper placeholder
    pub fn new(_cdc_class: CurrentCdcAcmClass) -> Self {
        Self {
            connected: false,
        }
    }
}

#[cfg(feature = "blackpill-f401")]
impl UsbCdc for UsbCdcWrapper {
    /// Write bytes to USB CDC
    async fn write(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("USB not connected");
        }

        match self.cdc_class.write_packet(data).await {
            Ok(_) => Ok(data.len()),
            Err(_) => {
                self.connected = false;
                Err("USB write failed")
            }
        }
    }

    /// Read bytes from USB CDC (non-blocking)
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.connected {
            return Err("USB not connected");
        }

        // Use a very short timeout to make it non-blocking
        match embassy_futures::select::select(
            self.cdc_class.read_packet(buffer),
            embassy_time::Timer::after_millis(1),
        )
        .await
        {
            embassy_futures::select::Either::First(result) => match result {
                Ok(len) => Ok(len),
                Err(_) => {
                    self.connected = false;
                    Err("USB read failed")
                }
            },
            embassy_futures::select::Either::Second(_) => {
                // Timeout - no data available
                Ok(0)
            }
        }
    }

    /// Check if USB CDC is connected and ready for communication
    fn is_connected(&self) -> bool {
        self.connected
    }

    /// Wait for USB CDC connection
    async fn wait_connection(&mut self) {
        self.cdc_class.wait_connection().await;
        self.connected = true;
    }
}

#[cfg(feature = "pipico")]
impl UsbCdc for UsbCdcWrapper {
    /// Write bytes to USB CDC (dummy implementation)
    async fn write(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.connected {
            // Consider as connected always in dummy implementation
            self.connected = true;
        }
        // Accept data and pretend it was written
        Ok(data.len())
    }

    /// Read bytes from USB CDC (dummy implementation, non-blocking)
    async fn read(&mut self, _buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.connected {
            self.connected = true;
        }
        // No data available in dummy implementation
        Ok(0)
    }

    /// Check if USB CDC is connected (dummy implementation)
    fn is_connected(&self) -> bool {
        self.connected
    }

    /// Wait for USB CDC connection (dummy implementation)
    async fn wait_connection(&mut self) {
        // Instantly consider connected
        self.connected = true;
    }
}
