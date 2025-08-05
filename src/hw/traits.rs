/// Hardware abstraction traits for core utilities
/// These traits provide a hardware-agnostic interface for debugging and device management

/// Type alias for initialization results that return a component and remaining peripherals
pub type InitResult<T> = Result<(T, embassy_stm32::Peripherals), &'static str>;

/// Device information structure
/// Contains hardware-specific information about the device
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub model: &'static str,
    pub board: &'static str,
    pub flash_size: u32,
    pub ram_size: u32,
    pub system_clock_hz: u32,
    pub usb_clock_hz: u32,
}

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
    /// USB Wrapper type that will be returned by init_usb
    type UsbWrapper;
    /// BackupRegisters type that will be returned by init_rtc
    type BackupRegisters: crate::hw::traits::BackupRegisters;

    /// Initialize the device with proper clock configuration
    /// This sets up the system clocks and returns the Embassy configuration
    fn init(&mut self) -> Result<embassy_stm32::Config, &'static str>;

    /// Check if the device has been initialized
    fn is_initialized(&self) -> bool;

    /// Get device information including model, board, memory sizes, and clock frequencies
    fn get_device_info(&self) -> DeviceInfo;

    /// Perform a soft reset of the device
    fn soft_reset(&self) -> !;

    /// Initialize LED peripheral separately for early debugging
    /// This method takes the full peripherals struct and extracts what it needs for LED initialization
    /// Returns initialized LED instance and remaining peripherals
    fn init_led(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Led>;

    /// Initialize USB peripheral from embassy_stm32::init output
    /// This method takes the peripherals struct and extracts what it needs for USB initialization
    /// Returns initialized USB wrapper instance and remaining peripherals
    fn init_usb(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> impl core::future::Future<Output = InitResult<Self::UsbWrapper>> + Send;

    /// Reboot the device normally
    /// This performs a standard system reset
    fn reboot(&self) -> !;

    /// Disable all interrupts to prevent interference during DFU transition
    /// This should disable both cortex-m interrupts and any hardware-specific interrupts
    fn disable_interrupts(&self);

    /// De-initialize the RTC peripheral
    /// This should reset the RTC to its default state and disable RTC clocking
    fn deinitialize_rtc(&self);

    /// De-initialize system clocks and prescalers
    /// This should reset the clock configuration to default state
    fn deinitialize_clocks(&self);

    /// Clear any pending interrupts
    /// This should clear all pending interrupts in the NVIC and other interrupt controllers
    fn clear_pending_interrupts(&self);

    /// Jump to the DFU bootloader without resetting the device
    /// This transfers control directly to the STM32 system DFU bootloader
    /// Note: This function will not return as it transfers control to the bootloader
    fn jump_to_dfu_bootloader(&self) -> !;

    /// Initialize a timer peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for timer initialization
    /// Returns initialized timer instance and remaining peripherals
    fn init_timer(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Timer>;

    /// Initialize an SPI peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for SPI initialization
    /// Returns initialized SPI instance and remaining peripherals
    fn init_spi(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Spi>;

    /// Initialize RTC peripheral and return backup registers wrapper
    /// This method takes the peripherals struct and extracts what it needs for RTC initialization
    /// Returns initialized backup registers instance and remaining peripherals
    fn init_rtc(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::BackupRegisters>;
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

/// Trait for abstracting backup register operations
/// Implementations should provide hardware-agnostic access to backup registers
/// that retain their values across system resets (but not power loss)
pub trait BackupRegisters {
    /// Read a u32 value from the specified backup register index
    fn read_register(&self, index: usize) -> u32;

    /// Write a u32 value to the specified backup register index
    fn write_register(&mut self, index: usize, value: u32);

    /// Get the number of available backup registers
    fn register_count(&self) -> usize;
}
