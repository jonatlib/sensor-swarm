/// Raspberry Pi Pico (RP2040) hardware implementation module
/// Contains platform-specific implementations for the Pi Pico board
pub mod backup_registers;
pub mod device;
pub mod flash;
pub mod gpio;
pub mod led;
pub mod usb;
pub mod usb_defmt_logger;

// Re-export commonly used types
pub use crate::hw::traits::DeviceInfo;
pub use backup_registers::PiPicoBackupRegisters;
pub use device::{init_embassy, PiPicoDevice};
pub use flash::{get_flash_range, PiPicoFlashStorage};
pub use gpio::{PiPicoGpioInit, PiPicoGpioManager, GpioPinInfo};
pub use led::{PiPicoLed, PiPicoLedManager, PiPicoPwmLed, LedInfo};
pub use usb::{CurrentCdcAcmClass, CurrentUsbDriver, CurrentUsbWrapper, UsbManager};

// Hardware-specific type aliases for Raspberry Pi Pico (RP2040)
/// Current device type - resolves to PiPicoDevice for pipico
pub type CurrentDevice = PiPicoDevice;

/// Current LED type - resolves to PiPicoLed for pipico
pub type CurrentLed = PiPicoLed;