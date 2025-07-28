/// STM32F401 Black Pill hardware implementation module
/// Contains platform-specific implementations for the Black Pill board

// pub mod usb;  // Temporarily disabled due to USB feature issues
pub mod gpio;

// pub use usb::UsbManager;  // Temporarily disabled
pub use gpio::{BlackPillLed, MockDeviceManager};