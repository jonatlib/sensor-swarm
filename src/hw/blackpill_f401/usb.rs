/// USB communication implementation for STM32F401 Black Pill
/// Provides hardware-specific USB byte-level communication and logging
/// Uses embassy-usb with CDC-ACM for serial communication over USB
use crate::hw::traits::{DebugInterface, UsbCommunication, UsbLogger};
use defmt::*;
use embassy_stm32::bind_interrupts;
use embassy_stm32::usb_otg::Driver;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::{Builder, Config, UsbDevice};
use heapless::String;

// Bind USB OTG FS interrupt
bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb_otg::InterruptHandler<embassy_stm32::peripherals::USB_OTG_FS>;
});

/// USB Communication Manager for STM32F401 Black Pill
/// Provides real USB CDC-ACM serial communication functionality
pub struct UsbManager {
    connected: bool,
    initialized: bool,
}

// Global USB components - these need to live for the entire program duration
static mut USB_DEVICE: Option<
    UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
> = None;
static mut CDC_CLASS: Option<
    CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
> = None;

impl UsbManager {
    /// Create a new USB Manager instance
    pub fn new() -> Self {
        Self {
            connected: false,
            initialized: false,
        }
    }

    /// Initialize USB peripheral with real USB functionality
    pub async fn init_with_peripheral(
        &mut self,
        usb: embassy_stm32::peripherals::USB_OTG_FS,
        dp: embassy_stm32::peripherals::PA12,
        dm: embassy_stm32::peripherals::PA11,
    ) -> Result<(), &'static str> {
        info!("Initializing USB CDC-ACM serial interface...");

        // Required buffers for USB driver and device
        static mut EP_OUT_BUFFER: [u8; 256] = [0; 256];
        static mut DEVICE_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONTROL_BUF: [u8; 64] = [0; 64];
        static mut MSOS_DESCRIPTOR: [u8; 256] = [0; 256];

        // Create USB OTG config
        let usb_config = embassy_stm32::usb_otg::Config::default();

        // Create the USB driver
        let driver = Driver::new_fs(usb, Irqs, dp, dm, unsafe { &mut EP_OUT_BUFFER }, usb_config);

        // Create USB device configuration
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Sensor Swarm");
        config.product = Some("STM32F401 Black Pill");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Create USB device builder
        let mut builder = Builder::new(
            driver,
            config,
            unsafe { &mut DEVICE_DESCRIPTOR },
            unsafe { &mut CONFIG_DESCRIPTOR },
            unsafe { &mut BOS_DESCRIPTOR },
            unsafe { &mut CONTROL_BUF },
            unsafe { &mut MSOS_DESCRIPTOR },
        );

        // Create CDC-ACM class with runtime state initialization
        use embassy_usb::class::cdc_acm::State;
        static mut STATE: Option<State> = None;

        // Initialize state at runtime
        let cdc_class = unsafe {
            STATE = Some(State::new());
            CdcAcmClass::new(&mut builder, STATE.as_mut().unwrap(), 64)
        };

        // Build the USB device
        let usb_device = builder.build();

        // Store the USB components in global statics
        unsafe {
            USB_DEVICE = Some(usb_device);
            CDC_CLASS = Some(cdc_class);
        }

        self.connected = true;
        self.initialized = true;

        info!("USB CDC-ACM serial interface initialized successfully");
        Ok(())
    }

    /// Run the USB device task (must be called continuously)
    pub async fn run_usb_task(&mut self) -> Result<(), &'static str> {
        unsafe {
            if let Some(ref mut usb_device) = USB_DEVICE {
                usb_device.run().await;
                Ok(())
            } else {
                Err("USB device not initialized")
            }
        }
    }
}

impl UsbCommunication for UsbManager {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<(), &'static str> {
        unsafe {
            if let Some(ref mut cdc) = CDC_CLASS {
                match cdc.write_packet(data).await {
                    Ok(_) => {
                        info!("Sent {} bytes over USB", data.len());
                        Ok(())
                    }
                    Err(_) => {
                        warn!("Failed to send {} bytes over USB", data.len());
                        Err("USB send failed")
                    }
                }
            } else {
                warn!("USB not initialized - cannot send {} bytes", data.len());
                Err("USB not initialized")
            }
        }
    }

    async fn receive_bytes(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        unsafe {
            if let Some(ref mut cdc) = CDC_CLASS {
                match cdc.read_packet(buffer).await {
                    Ok(len) => {
                        info!("Received {} bytes over USB", len);
                        Ok(len)
                    }
                    Err(_) => {
                        warn!("Failed to receive bytes over USB");
                        Err("USB receive failed")
                    }
                }
            } else {
                warn!("USB not initialized - cannot receive bytes");
                Err("USB not initialized")
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl UsbLogger for UsbManager {
    async fn log(&mut self, message: &str) -> Result<(), &'static str> {
        // Use heapless string for no_std environment
        let mut log_msg = String::<512>::new();
        match core::fmt::write(&mut log_msg, format_args!("[USB] {message}\r\n")) {
            Ok(_) => self.send_bytes(log_msg.as_bytes()).await,
            Err(_) => {
                error!("Failed to format USB log message");
                Err("USB log formatting failed")
            }
        }
    }

    async fn log_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result<(), &'static str> {
        // Format the message into a heapless string (limited to 256 chars)
        let mut formatted = String::<256>::new();
        match core::fmt::write(&mut formatted, args) {
            Ok(_) => self.log(formatted.as_str()).await,
            Err(_) => {
                error!("Failed to format log message");
                Err("Log message formatting failed")
            }
        }
    }
}

impl DebugInterface for UsbManager {
    /// Initialize the debug interface
    async fn init(&mut self) -> Result<(), &'static str> {
        info!("USB debug interface initialized");
        Ok(())
    }
}

impl Default for UsbManager {
    fn default() -> Self {
        Self::new()
    }
}
