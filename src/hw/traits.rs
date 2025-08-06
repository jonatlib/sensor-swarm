/// Hardware abstraction traits for core utilities
/// These traits provide a hardware-agnostic interface for debugging and device management

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
    pub unique_id_hex: heapless::String<24>,
}

/// Trait for abstracting debug interface setup
/// Implementations should configure the appropriate logging backend (USB, RTT, etc.)
pub trait DebugInterface {
    /// Initialize the debug interface
    /// This should set up the logging backend and make it ready for use
    fn init(&mut self) -> impl core::future::Future<Output = Result<(), &'static str>> + Send;
}

/// Trait for abstracting device management operations with safe peripheral handling
/// This trait uses lifetimes to bind peripherals safely to the device manager,
/// eliminating the need for unsafe pointer operations
pub trait DeviceManagement<'d> {
    /// LED type that will be created from stored peripherals
    type Led: crate::hw::traits::Led;
    /// USB Wrapper type that will be created from stored peripherals
    type UsbWrapper;
    /// BackupRegisters type that will be created from stored peripherals
    type BackupRegisters: crate::hw::traits::BackupRegisters;

    /// Create a new device manager instance with peripherals
    /// This static method returns the Embassy configuration and creates the device manager
    /// with the peripherals stored internally
    fn new_with_peripherals(peripherals: embassy_stm32::Peripherals) -> Result<(embassy_stm32::Config, Self), &'static str>
    where
        Self: Sized;

    /// Get device information including model, board, memory sizes, and clock frequencies
    fn get_device_info(&self) -> DeviceInfo;

    /// Perform a soft reset of the device
    fn soft_reset(&self) -> !;

    /// Create LED peripheral from stored peripherals for early debugging
    /// This method uses the internally stored peripherals to create an LED instance
    /// The LED is bound to the device manager's lifetime
    fn create_led(&'d mut self) -> Result<Self::Led, &'static str>;

    /// Create USB peripheral from stored peripherals
    /// This method uses the internally stored peripherals to create a USB wrapper instance
    /// The USB wrapper is bound to the device manager's lifetime
    fn create_usb(&'d mut self) -> impl core::future::Future<Output = Result<Self::UsbWrapper, &'static str>> + Send;

    /// Create RTC peripheral and backup registers from stored peripherals
    /// This method uses the internally stored peripherals to create backup registers
    /// The backup registers are bound to the device manager's lifetime
    fn create_rtc(&'d mut self) -> Result<Self::BackupRegisters, &'static str>;

    /// Get access to backup registers for boot task management
    /// This method provides access to backup registers that have been created via create_rtc
    /// Returns None if backup registers haven't been created yet
    fn get_backup_registers(&mut self) -> Option<&mut Self::BackupRegisters>;

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

    /// Get the unique hardware ID as a byte array
    /// Returns the device's unique identifier as raw bytes
    fn get_unique_id_bytes(&self) -> [u8; 12];

    /// Get the unique hardware ID as a hexadecimal string
    /// Returns the device's unique identifier formatted as a hex string
    fn get_unique_id_hex(&self) -> heapless::String<24>;
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
