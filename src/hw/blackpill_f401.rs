/// STM32F401 Black Pill hardware implementation module
/// Contains platform-specific implementations for the Black Pill board
pub mod backup_registers;
pub mod device;
pub mod flash;
pub mod gpio;
pub mod led;
pub mod usb;
pub mod usb_defmt_logger;

// Re-export commonly used types
pub use crate::hw::traits::DeviceInfo;
pub use backup_registers::BlackPillBackupRegisters;
pub use device::{init_embassy, BlackPillDevice};
pub use flash::{get_eeprom_range, EepromStorage};
pub use gpio::{BlackPillGpioInit, BlackPillGpioManager, GpioPinInfo};
pub use led::{BlackPillLed, BlackPillLedManager, BlackPillPwmLed, LedInfo};
pub use usb::{CurrentCdcAcmClass, CurrentUsbDriver, CurrentUsbWrapper, UsbManager};

// Hardware-specific type aliases for STM32F401 Black Pill
/// Current device type - resolves to BlackPillDevice for blackpill-f401
pub type CurrentDevice = BlackPillDevice;

/// Current LED type - resolves to BlackPillLed for blackpill-f401
pub type CurrentLed = BlackPillLed;
