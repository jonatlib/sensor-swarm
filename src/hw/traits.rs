/// Hardware abstraction traits for core utilities
/// These traits provide a hardware-agnostic interface for debugging and device management

/// Trait for abstracting debug interface setup
/// Implementations should configure the appropriate logging backend (USB, RTT, etc.)
pub trait DebugInterface {
    /// Initialize the debug interface
    /// This should set up the logging backend and make it ready for use
    fn init(&mut self) -> impl core::future::Future<Output = Result<(), &'static str>> + Send;
}

/// Trait for abstracting device management operations
/// Implementations should provide platform-specific device control functions
pub trait DeviceManagement {
    /// Reboot the device into the DFU bootloader
    /// This allows for easy firmware updates via USB DFU
    fn reboot_to_bootloader(&self) -> !;
}

/// Trait for abstracting GPIO operations
/// Implementations should provide hardware-agnostic GPIO control
pub trait GpioPin {
    /// Set the pin to high level
    fn set_high(&mut self);
    
    /// Set the pin to low level
    fn set_low(&mut self);
    
    /// Toggle the pin state
    fn toggle(&mut self);
}

/// Trait for abstracting LED operations
/// Implementations should provide hardware-agnostic LED control
pub trait Led {
    /// Turn the LED on
    fn on(&mut self);
    
    /// Turn the LED off
    fn off(&mut self);
    
    /// Toggle the LED state
    fn toggle(&mut self);
}