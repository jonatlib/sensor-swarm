/// STM32F401 Black Pill hardware implementation module
/// Contains platform-specific implementations for the Black Pill board

pub mod usb;

pub use usb::UsbManager;