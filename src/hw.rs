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
#[cfg(feature = "blackpill-f401")]
pub use blackpill_f401 as device_module;

/// Raspberry Pi Pico (RP2040) implementation
#[cfg(feature = "pipico")]
pub mod pipico;
#[cfg(feature = "pipico")]
pub use pipico as device_module;

// Re-export commonly used items
pub use traits::*;
pub use types::*;

// Conditional type aliases for unified device access
// This allows main.rs to use a single CurrentDevice type regardless of hardware platform

/// Current device type - resolves based on the selected device module
pub use device_module::CurrentDevice;

/// Current LED type - resolves based on the selected device module
pub use device_module::CurrentLed;

/// Current USB wrapper type - resolves based on the selected device module
pub use device_module::CurrentUsbWrapper;

/// Current USB driver type - resolves based on the selected device module
pub use device_module::CurrentUsbDriver;

/// Current CDC ACM class type - resolves based on the selected device module
pub use device_module::CurrentCdcAcmClass;

/// Embassy initialization function - resolves based on the selected device module
pub use device_module::init_embassy;

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
