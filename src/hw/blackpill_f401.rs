/// STM32F401 Black Pill hardware implementation module
/// Contains platform-specific implementations for the Black Pill board

pub mod device;
pub mod gpio;
pub mod led;
pub mod flash;
pub mod usb;

// Re-export commonly used types
pub use device::{BlackPillDevice, DeviceInfo};
pub use gpio::{BlackPillGpioInit, BlackPillGpioManager, GpioPinInfo};
pub use led::{BlackPillLed, BlackPillPwmLed, BlackPillLedManager, LedInfo};
pub use flash::{BlackPillFlashStorage, BlackPillFlashManager, BlackPillKeyValueStore, FlashInfo, FlashStorageInfo};
pub use usb::UsbManager;