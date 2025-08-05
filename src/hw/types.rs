/// Types for backup register management
/// This module defines the enums and types used for managing backup registers
/// and boot tasks in a hardware-agnostic way

/// Enum to define which backup register we are using for a specific purpose.
#[repr(usize)]
pub enum BackupRegister {
    /// Stores the action to be performed after a reboot.
    BootTask = 0,
    /// Could be used for something else, e.g., storing a boot count.
    BootCounter = 1,
}

/// Enum for the specific task to be performed after boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
#[repr(u32)]
pub enum BootTask {
    /// Default state: do nothing special.
    None = 0,
    /// A task to update the firmware.
    UpdateFirmware, // Will be 1
    /// A task to run a system self-test.
    RunSelfTest,    // Will be 2
}

/// Safely converts a raw u32 value from the register into a BootTask.
impl From<u32> for BootTask {
    fn from(value: u32) -> Self {
        match value {
            1 => BootTask::UpdateFirmware,
            2 => BootTask::RunSelfTest,
            _ => BootTask::None,
        }
    }
}
