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

/// USB wrapper that owns the USB device and CDC class components
/// This struct encapsulates the low-level USB components and provides high-level interface
pub struct UsbWrapper {
    usb_device: UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    connected: bool,
}

impl UsbWrapper {
    /// Create a new USB wrapper with the given components
    pub fn new(
        usb_device: UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
        cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    ) -> Self {
        Self {
            usb_device,
            cdc_class,
            connected: false,
        }
    }

    /// Run the USB device task (should be called continuously)
    pub async fn run_usb_device(&mut self) {
        self.usb_device.run().await;
    }

    /// Handle CDC communication (echo and logging)
    pub async fn handle_cdc_communication(&mut self) -> Result<(), &'static str> {
        // Wait for connection
        self.cdc_class.wait_connection().await;
        self.connected = true;
        info!("USB CDC connected!");

        // Handle communication until disconnected
        let mut buf = [0; 64];
        loop {
            match self.cdc_class.read_packet(&mut buf).await {
                Ok(len) if len > 0 => {
                    info!("Received {} bytes over USB: {:?}", len, &buf[..len]);
                    // Echo back the received data
                    let _ = self.cdc_class.write_packet(&buf[..len]).await;
                }
                Ok(_) => {
                    // No data received, continue
                }
                Err(_) => {
                    info!("USB CDC disconnected");
                    self.connected = false;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Send data over USB CDC
    pub async fn send_data(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if self.connected {
            match self.cdc_class.write_packet(data).await {
                Ok(_) => Ok(()),
                Err(_) => Err("Failed to send data over USB"),
            }
        } else {
            Err("USB not connected")
        }
    }

    /// Send log message over USB
    pub async fn send_log(&mut self, message: &str) -> Result<(), &'static str> {
        if self.connected {
            // Use heapless string for no_std environment
            let mut log_msg = String::<512>::new();
            match core::fmt::write(&mut log_msg, format_args!("[LOG] {}\r\n", message)) {
                Ok(_) => self.send_data(log_msg.as_bytes()).await,
                Err(_) => Err("Failed to format log message"),
            }
        } else {
            Ok(()) // Silently ignore if not connected
        }
    }

    /// Check if USB is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

// USB components are now returned directly from init_with_peripheral
// No global statics needed - Embassy tasks will own the components

impl UsbManager {
    /// Create a new USB Manager instance
    pub fn new() -> Self {
        Self {
            connected: false,
            initialized: false,
        }
    }

    /// Initialize USB peripheral with real USB functionality
    /// Returns a USB wrapper that owns the USB device and CDC class
    pub async fn init_with_peripheral(
        &mut self,
        usb: embassy_stm32::peripherals::USB_OTG_FS,
        dp: embassy_stm32::peripherals::PA12,
        dm: embassy_stm32::peripherals::PA11,
    ) -> Result<UsbWrapper, &'static str> {
        info!("Initializing USB CDC-ACM serial interface...");

        // Required buffers for USB driver and device
        static mut EP_OUT_BUFFER: [u8; 256] = [0; 256];
        static mut DEVICE_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONTROL_BUF: [u8; 64] = [0; 64];
        static mut MSOS_DESCRIPTOR: [u8; 256] = [0; 256];

        // Create USB OTG config with proper settings for STM32F401
        let mut usb_config = embassy_stm32::usb_otg::Config::default();
        // Do not enable vbus_detection. This is a safe default that works in all boards.
        usb_config.vbus_detection = false;

        // Create the USB driver
        let driver = Driver::new_fs(usb, Irqs, dp, dm, unsafe { &mut EP_OUT_BUFFER }, usb_config);

        // Create USB device configuration - using working example VID/PID
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Set device class to CDC (Communications Device Class) for proper serial port recognition
        config.device_class = 0x02; // CDC class
        config.device_sub_class = 0x00;
        config.device_protocol = 0x00;

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

        self.connected = false; // Will be set to true when USB is actually enumerated
        self.initialized = true;

        info!("USB CDC-ACM serial interface initialized successfully");
        info!("USB wrapper ready for task execution");
        Ok(UsbWrapper::new(usb_device, cdc_class))
    }
}

// Minimal trait implementations for compatibility - actual communication handled by tasks
impl UsbCommunication for UsbManager {
    async fn send_bytes(&mut self, _data: &[u8]) -> Result<(), &'static str> {
        Err("USB communication should be handled by tasks that own the CDC class")
    }

    async fn receive_bytes(&mut self, _buffer: &mut [u8]) -> Result<usize, &'static str> {
        Err("USB communication should be handled by tasks that own the CDC class")
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl UsbLogger for UsbManager {
    async fn log(&mut self, _message: &str) -> Result<(), &'static str> {
        Err("USB logging should be handled by tasks that own the CDC class")
    }

    async fn log_fmt(&mut self, _args: core::fmt::Arguments<'_>) -> Result<(), &'static str> {
        Err("USB logging should be handled by tasks that own the CDC class")
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
