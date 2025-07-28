/// Hardware abstraction traits for core utilities
/// These traits provide a hardware-agnostic interface for debugging and device management

/// Trait for abstracting debug interface setup
/// Implementations should configure the appropriate logging backend (USB, RTT, etc.)
pub trait DebugInterface {
    /// Initialize the debug interface
    /// This should set up the logging backend and make it ready for use
    async fn init(&mut self) -> Result<(), &'static str>;
}

/// Trait for abstracting device management operations
/// Implementations should provide platform-specific device control functions
pub trait DeviceManagement {
    /// Reboot the device into the DFU bootloader
    /// This allows for easy firmware updates via USB DFU
    fn reboot_to_bootloader(&self) -> !;
}