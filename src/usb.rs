/// Hardware-agnostic USB communication module
/// This module provides USB CDC communication functionality that is independent of specific hardware implementations
/// The UsbManager for hardware-specific initialization remains in the hw module
use embassy_stm32::usb::Driver;
use embassy_usb::class::cdc_acm::CdcAcmClass;

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

/// Simple USB CDC wrapper that implements the UsbCdc trait
/// This struct provides basic read/write operations for USB CDC communication
pub struct UsbCdcWrapper {
    cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    connected: bool,
}

impl UsbCdcWrapper {
    /// Create a new USB CDC wrapper with the given CDC class
    pub fn new(
        cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    ) -> Self {
        Self {
            cdc_class,
            connected: false,
        }
    }
}

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
