/// Hardware abstraction module
/// This module contains all direct hardware interaction and platform-specific code
/// All hardware access must be isolated in this module to maintain hardware abstraction

/// Hardware abstraction traits
pub mod traits;

/// Types for backup register management
pub mod types;

/// STM32F401 Black Pill implementation
#[cfg(feature = "blackpill-f401")]
pub mod blackpill_f401;

/// Raspberry Pi Pico (RP2040) implementation
#[cfg(feature = "pipico")]
pub mod pipico;

// Re-export commonly used items
pub use traits::*;
pub use types::*;

// Conditional type aliases for unified device access
// This allows main.rs to use a single CurrentDevice type regardless of hardware platform

#[cfg(feature = "blackpill-f401")]
/// Current device type - resolves to BlackPillDevice when blackpill-f401 feature is enabled
pub type CurrentDevice = BlackPillDevice;

#[cfg(feature = "pipico")]
/// Current device type - resolves to PiPicoDevice when pipico feature is enabled
pub type CurrentDevice = PiPicoDevice;

#[cfg(feature = "blackpill-f401")]
/// Current LED type - resolves to BlackPillLed when blackpill-f401 feature is enabled
pub type CurrentLed = BlackPillLed;

#[cfg(feature = "pipico")]
/// Current LED type - resolves to PiPicoLed when pipico feature is enabled
pub type CurrentLed = PiPicoLed;

#[cfg(feature = "blackpill-f401")]
/// Current USB wrapper type - resolves to UsbCdcWrapper when blackpill-f401 feature is enabled
pub type CurrentUsbWrapper = crate::usb::UsbCdcWrapper;

#[cfg(feature = "pipico")]
/// Current USB wrapper type - resolves to unit type when pipico feature is enabled (USB not implemented yet)
pub type CurrentUsbWrapper = ();

// Embassy initialization functions
#[cfg(feature = "blackpill-f401")]
/// Initialize embassy with current device configuration
pub fn init_embassy() -> embassy_stm32::Peripherals {
    embassy_stm32::init(BlackPillDevice::get_embassy_config())
}

#[cfg(feature = "pipico")]
/// Initialize embassy with current device configuration
pub fn init_embassy() -> embassy_rp::Peripherals {
    embassy_rp::init(PiPicoDevice::get_embassy_config())
}

#[cfg(feature = "blackpill-f401")]
pub use blackpill_f401::{
    get_eeprom_range,
    // Device management
    BlackPillDevice,
    // GPIO
    BlackPillGpioInit,
    BlackPillGpioManager,
    // LED with PWM support
    BlackPillLed,
    BlackPillLedManager,
    BlackPillPwmLed,
    DeviceInfo,
    // Flash/EEPROM
    EepromStorage,
    GpioPinInfo,
    LedInfo,
    // USB Communication
    UsbManager,
};

#[cfg(feature = "pipico")]
pub use pipico::{
    get_flash_range,
    // Device management
    PiPicoDevice,
    // GPIO
    PiPicoGpioInit,
    PiPicoGpioManager,
    // LED with PWM support
    PiPicoLed,
    PiPicoLedManager,
    PiPicoPwmLed,
    DeviceInfo,
    // Flash storage
    PiPicoFlashStorage,
    GpioPinInfo,
    LedInfo,
    // USB Communication
    UsbManager,
};
