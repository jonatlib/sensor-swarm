// Hardware abstraction module
// This module contains all direct hardware interaction and platform-specific code
// All hardware access must be isolated in this module to maintain hardware abstraction

/// Hardware abstraction traits
pub mod traits;

/// STM32F401 Black Pill implementation
#[cfg(feature = "blackpill-f401")]
pub mod blackpill_f401;

// Re-export commonly used items
pub use traits::*;

#[cfg(feature = "blackpill-f401")]
pub use blackpill_f401::{
    // Device management
    BlackPillDevice, DeviceInfo,
    // GPIO
    BlackPillGpioInit, BlackPillGpioManager, GpioPinInfo,
    // LED with PWM support
    BlackPillLed, BlackPillPwmLed, BlackPillLedManager, LedInfo,
    // Flash/EEPROM
    BlackPillFlashStorage, BlackPillFlashManager, BlackPillKeyValueStore, FlashInfo, FlashStorageInfo,
    // USB Communication
    UsbManager,
};