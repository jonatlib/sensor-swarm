/// USB communication implementation for STM32F401 Black Pill
/// Hardware-specific USB initialization and management
use crate::usb::UsbCdcWrapper;
use defmt::*;
use embassy_stm32::bind_interrupts;
use embassy_stm32::usb::{Config as UsbConfig, Driver};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::{Builder, Config};

// Bind USB OTG FS interrupt
bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB_OTG_FS>;
});

/// USB Communication Manager for STM32F401 Black Pill
/// Provides real USB CDC-ACM serial communication functionality
pub struct UsbManager {
    initialized: bool,
}

// USB components are now returned directly from init_with_peripheral
// No global statics needed - Embassy tasks will own the components

impl UsbManager {
    /// Create a new USB Manager instance
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize USB peripheral with real USB functionality
    /// Returns a USB CDC wrapper that implements the UsbCdc trait
    pub async fn init_with_peripheral(
        &mut self,
        usb: embassy_stm32::peripherals::USB_OTG_FS,
        dp: embassy_stm32::peripherals::PA12,
        dm: embassy_stm32::peripherals::PA11,
    ) -> Result<UsbCdcWrapper, &'static str> {
        info!("Initializing USB CDC-ACM serial interface...");

        // TODO: Consider safer buffer management for production
        // These static mutable buffers could be replaced with safer alternatives
        // Required buffers for USB driver and device
        static mut EP_OUT_BUFFER: [u8; 256] = [0; 256];
        static mut DEVICE_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
        static mut CONTROL_BUF: [u8; 64] = [0; 64];
        static mut MSOS_DESCRIPTOR: [u8; 256] = [0; 256];

        // Create USB OTG config with proper settings for STM32F401
        let mut usb_config = UsbConfig::default();
        // Do not enable vbus_detection. This is a safe default that works in all boards.
        usb_config.vbus_detection = false;

        // Create the USB driver
        let driver = Driver::new_fs(usb, Irqs, dp, dm, unsafe { &mut EP_OUT_BUFFER }, usb_config);

        // TODO: Replace hardcoded USB device configuration with production values
        // - Use proper VID/PID registered for the product
        // - Set appropriate manufacturer, product name, and serial number
        // - Make configuration configurable or read from device-specific storage
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
        );

        // TODO: Consider safer state management for production
        // This unsafe static initialization could be replaced with safer alternatives
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

        self.initialized = true;

        info!("USB CDC-ACM serial interface initialized successfully");
        info!("USB CDC wrapper ready for task execution");
        Ok(UsbCdcWrapper::new(cdc_class))
    }
}

impl Default for UsbManager {
    fn default() -> Self {
        Self::new()
    }
}
