/// STM32F401 Black Pill hardware implementation module
/// Contains platform-specific implementations for the Black Pill board
pub mod device;
pub mod flash;
pub mod gpio;
pub mod led;
pub mod usb;
pub mod usb_defmt_logger;

// Re-export commonly used types
pub use device::{BlackPillDevice, DeviceInfo};
pub use flash::{EepromStorage, get_eeprom_range};
pub use gpio::{BlackPillGpioInit, BlackPillGpioManager, GpioPinInfo};
pub use led::{BlackPillLed, BlackPillLedManager, BlackPillPwmLed, LedInfo};
pub use usb::UsbManager;
