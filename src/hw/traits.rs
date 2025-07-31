/// Hardware abstraction traits for core utilities
/// These traits provide a hardware-agnostic interface for debugging and device management

/// Type alias for initialization results that return a component and remaining peripherals
pub type InitResult<T> = Result<(T, embassy_stm32::Peripherals), &'static str>;

/// Trait for abstracting debug interface setup
/// Implementations should configure the appropriate logging backend (USB, RTT, etc.)
pub trait DebugInterface {
    /// Initialize the debug interface
    /// This should set up the logging backend and make it ready for use
    fn init(&mut self) -> impl core::future::Future<Output = Result<(), &'static str>> + Send;
}

/// Trait for abstracting device management operations
/// Implementations should provide platform-specific device control functions
pub trait DeviceManagement {
    /// Timer peripheral type that will be returned by init_timer
    type Timer: embassy_stm32::Peripheral;
    /// SPI peripheral type that will be returned by init_spi
    type Spi: embassy_stm32::Peripheral;
    /// LED type that will be returned by init_peripherals
    type Led: crate::hw::traits::Led;
    /// USB Manager type that will be returned by init_peripherals
    type UsbManager: crate::hw::traits::UsbCommunication + crate::hw::traits::UsbLogger;

    /// Initialize LED peripheral separately for early debugging
    /// This method takes the full peripherals struct and extracts what it needs for LED initialization
    /// Returns initialized LED instance and remaining peripherals
    fn init_led(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> InitResult<Self::Led>;

    /// Initialize USB peripheral from embassy_stm32::init output
    /// This method takes the peripherals struct and extracts what it needs for USB initialization
    /// Returns initialized USB manager instance and remaining peripherals
    fn init_usb(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> impl core::future::Future<Output = InitResult<Self::UsbManager>> + Send;

    /// Reboot the device normally
    /// This performs a standard system reset
    fn reboot(&self) -> !;

    /// Reboot the device into the DFU bootloader
    /// This allows for easy firmware updates via USB DFU
    fn reboot_to_bootloader(&self) -> !;

    /// Initialize a timer peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for timer initialization
    /// Returns initialized timer instance and remaining peripherals
    fn init_timer(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> InitResult<Self::Timer>;

    /// Initialize an SPI peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for SPI initialization
    /// Returns initialized SPI instance and remaining peripherals
    fn init_spi(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> InitResult<Self::Spi>;
}

// GPIO functionality is provided directly by Embassy GPIO types (Output, Input)
// No custom trait needed - use embassy_stm32::gpio::{Output, Input} directly

/// Trait for abstracting LED operations
/// Implementations should provide hardware-agnostic LED control with PWM support
pub trait Led {
    /// Turn the LED on
    fn on(&mut self);

    /// Turn the LED off
    fn off(&mut self);

    /// Toggle the LED state
    fn toggle(&mut self);

    /// Set LED brightness using PWM (0-255, where 0 is off and 255 is full brightness)
    fn set_brightness(&mut self, brightness: u8);
}

/// Trait for abstracting USB communication at byte level
/// Implementations should provide hardware-agnostic USB byte send/receive
pub trait UsbCommunication {
    /// Send bytes over USB
    fn send_bytes(
        &mut self,
        data: &[u8],
    ) -> impl core::future::Future<Output = Result<(), &'static str>>;

    /// Receive bytes from USB (non-blocking)
    fn receive_bytes(
        &mut self,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = Result<usize, &'static str>>;

    /// Check if USB is connected and ready
    fn is_connected(&self) -> bool;
}

/// Trait for abstracting USB logging functionality
/// Implementations should provide hardware-agnostic logging over USB serial
pub trait UsbLogger {
    /// Log a message over USB serial
    fn log(
        &mut self,
        message: &str,
    ) -> impl core::future::Future<Output = Result<(), &'static str>>;

    /// Log formatted message over USB serial
    fn log_fmt(
        &mut self,
        args: core::fmt::Arguments<'_>,
    ) -> impl core::future::Future<Output = Result<(), &'static str>>;
}

// Timer functionality is provided directly by Embassy Timer (embassy_time::Timer)
// No custom trait needed - use embassy_time::{Timer, Duration, Instant} directly

// SPI functionality is provided directly by Embassy SPI traits
// No custom trait needed - use embassy_stm32::spi::Spi and related traits directly

/// Trait for abstracting Flash/EEPROM operations
/// Implementations should provide hardware-agnostic persistent storage
pub trait FlashStorage {
    /// Read data from flash at specified address
    fn read(&self, address: u32, buffer: &mut [u8]) -> Result<(), &'static str>;

    /// Write data to flash at specified address
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), &'static str>;

    /// Erase flash sector containing the specified address
    fn erase_sector(&mut self, address: u32) -> Result<(), &'static str>;

    /// Get the size of a flash sector
    fn sector_size(&self) -> u32;

    /// Get the total flash size available for storage
    fn total_size(&self) -> u32;
}
