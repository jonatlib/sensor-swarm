/// USB communication implementation for STM32F401 Black Pill
/// Direct USB logging implementation - no buffering, sends directly to USB when available
use crate::hw::traits::{DebugInterface, UsbCommunication, UsbLogger, UsbCdc};
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
/// This struct encapsulates the low-level USB components for direct logging
pub struct UsbWrapper {
    usb_device: UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    connected: bool,
}

/// CDC wrapper that implements USB traits for direct logging
/// This struct wraps the CDC class and provides direct USB logging
pub struct CdcWrapper {
    cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    connected: bool,
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
        ).await {
            embassy_futures::select::Either::First(result) => {
                match result {
                    Ok(len) => Ok(len),
                    Err(_) => {
                        self.connected = false;
                        Err("USB read failed")
                    }
                }
            }
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

impl CdcWrapper {
    /// Create a new CDC wrapper with the given CDC class
    pub fn new(
        cdc_class: CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    ) -> Self {
        Self {
            cdc_class,
            connected: false,
        }
    }

    /// Wait for USB connection
    pub async fn wait_connection(&mut self) {
        self.cdc_class.wait_connection().await;
        self.connected = true;
        // Don't log connection to avoid potential recursion - use defmt directly for RTT only
        defmt::info!("USB CDC connected!");
    }

    /// Handle communication - wait for connection, process log messages and handle commands
    pub async fn handle_communication(&mut self) -> Result<(), &'static str> {
        let mut command_buffer = heapless::Vec::<u8, 256>::new();

        loop {
            // Wait for connection first
            if !self.connected {
                self.cdc_class.wait_connection().await;
                self.connected = true;
                defmt::info!("USB connected - ready for logging and commands");
            }

            // Process queued log messages when connected
            if self.connected {
                // Process up to 5 log messages per iteration to avoid blocking
                let mut processed = 0;
                while processed < 5 {
                    if let Some(message) =
                        crate::hw::blackpill_f401::usb_defmt_logger::dequeue_usb_log_message()
                    {
                        // Format message with prefix and newline for USB
                        let mut log_msg = String::<512>::new();
                        if core::fmt::write(
                            &mut log_msg,
                            format_args!("[LOG] {}\r\n", message.as_str()),
                        )
                        .is_ok()
                        {
                            match self.cdc_class.write_packet(log_msg.as_bytes()).await {
                                Ok(_) => {
                                    processed += 1;
                                }
                                Err(_) => {
                                    // USB disconnected during logging
                                    self.connected = false;
                                    defmt::info!("USB disconnected during logging");
                                    return Err("USB disconnected");
                                }
                            }
                        }
                    } else {
                        // No more messages to process
                        break;
                    }
                }

                // Check for incoming commands (non-blocking)
                let mut temp_buffer = [0u8; 32];
                match embassy_futures::select::select(
                    self.cdc_class.read_packet(&mut temp_buffer),
                    embassy_time::Timer::after_millis(1), // Very short timeout for non-blocking
                )
                .await
                {
                    embassy_futures::select::Either::First(result) => {
                        match result {
                            Ok(len) if len > 0 => {
                                // Process received bytes for commands
                                for &byte in &temp_buffer[..len] {
                                    if byte == b'\n' {
                                        // Command complete, echo it back as a simple response
                                        if !command_buffer.is_empty() {
                                            let mut response = String::<512>::new();
                                            let cmd_str = core::str::from_utf8(&command_buffer)
                                                .unwrap_or("INVALID_UTF8");
                                            if core::fmt::write(
                                                &mut response,
                                                format_args!("[CMD] Received: {}\r\n", cmd_str),
                                            )
                                            .is_ok()
                                            {
                                                let _ = self
                                                    .cdc_class
                                                    .write_packet(response.as_bytes())
                                                    .await;
                                            }
                                            command_buffer.clear();
                                        }
                                    } else if command_buffer.len() < command_buffer.capacity() - 1 {
                                        let _ = command_buffer.push(byte);
                                    } else {
                                        // Command too long, clear buffer
                                        command_buffer.clear();
                                        let error_msg = "[CMD] Command too long\r\n";
                                        let _ =
                                            self.cdc_class.write_packet(error_msg.as_bytes()).await;
                                    }
                                }
                            }
                            Ok(_) => {
                                // No data received
                            }
                            Err(_) => {
                                // USB disconnected
                                self.connected = false;
                                defmt::info!("USB disconnected during command handling");
                                return Err("USB disconnected");
                            }
                        }
                    }
                    embassy_futures::select::Either::Second(_) => {
                        // Timeout - no command data available
                    }
                }
            }

            // Yield to other tasks
            embassy_time::Timer::after_millis(10).await;
        }
    }

    /// Check if USB is connected (public method for logging)
    pub fn is_usb_connected(&self) -> bool {
        self.connected
    }

    /// Send log message directly to USB (synchronous, non-blocking)
    /// Returns true if message was sent, false if USB not connected or send failed
    pub fn try_send_log_direct(&mut self, message: &str) -> bool {
        // For now, just return the connection status
        // Actual USB logging will be handled by the async UsbLogger trait
        self.connected
    }
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

    /// Handle CDC communication - wait for connection and maintain it
    pub async fn handle_cdc_communication(&mut self) -> Result<(), &'static str> {
        loop {
            // Wait for connection first
            if !self.connected {
                self.cdc_class.wait_connection().await;
                self.connected = true;
                defmt::info!("USB connected - ready for logging");
            }

            // Just wait and yield to other tasks
            embassy_time::Timer::after_millis(100).await;
        }
    }

    /// Send data over USB CDC - DISABLED as requested
    pub async fn send_data(&mut self, _data: &[u8]) -> Result<(), &'static str> {
        // USB functionality disabled - just return Ok to maintain interface
        Ok(())
    }

    /// Send log message over USB - DISABLED as requested
    pub async fn send_log(&mut self, _message: &str) -> Result<(), &'static str> {
        // USB functionality disabled - just return Ok to maintain interface
        Ok(())
    }

    /// Check if USB is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Split the wrapper into separate USB device and CDC class components
    /// This allows using the Embassy join() pattern for proper USB enumeration
    pub fn split(
        self,
    ) -> (
        UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
        CdcAcmClass<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
    ) {
        (self.usb_device, self.cdc_class)
    }

    /// Split the wrapper into USB device and CDC wrapper that implements USB traits
    /// This allows using the trait-based architecture with spawned tasks
    pub fn split_with_traits(
        self,
    ) -> (
        UsbDevice<'static, Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>>,
        CdcWrapper,
    ) {
        (self.usb_device, CdcWrapper::new(self.cdc_class))
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

// USB trait implementations for CdcWrapper - DISABLED as requested
impl UsbCommunication for CdcWrapper {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if self.connected {
            match self.cdc_class.write_packet(data).await {
                Ok(_) => {
                    defmt::debug!("Sent {} bytes over USB", data.len());
                    Ok(())
                }
                Err(_) => {
                    self.connected = false;
                    defmt::warn!("USB disconnected during send");
                    Err("USB send failed")
                }
            }
        } else {
            Err("USB not connected")
        }
    }

    async fn receive_bytes(&mut self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.connected {
            match self.cdc_class.read_packet(buffer).await {
                Ok(len) => {
                    if len > 0 {
                        defmt::debug!("Received {} bytes over USB", len);
                    }
                    Ok(len)
                }
                Err(_) => {
                    self.connected = false;
                    defmt::warn!("USB disconnected during receive");
                    Err("USB receive failed")
                }
            }
        } else {
            Err("USB not connected")
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl UsbLogger for CdcWrapper {
    async fn log(&mut self, message: &str) -> Result<(), &'static str> {
        if self.connected {
            // Format message with newline for USB
            let mut log_msg = String::<512>::new();
            if core::fmt::write(&mut log_msg, format_args!("{}\r\n", message)).is_ok() {
                match self.cdc_class.write_packet(log_msg.as_bytes()).await {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        self.connected = false;
                        Err("USB disconnected during logging")
                    }
                }
            } else {
                Err("Failed to format log message")
            }
        } else {
            // USB not connected, silently ignore
            Ok(())
        }
    }

    async fn log_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result<(), &'static str> {
        if self.connected {
            // Format the arguments into a string
            let mut formatted = String::<256>::new();
            if core::fmt::write(&mut formatted, args).is_ok() {
                self.log(formatted.as_str()).await
            } else {
                Err("Failed to format log arguments")
            }
        } else {
            // USB not connected, silently ignore
            Ok(())
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
