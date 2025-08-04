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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_task_from_u32() {
        assert_eq!(BootTask::from(0), BootTask::None);
        assert_eq!(BootTask::from(1), BootTask::UpdateFirmware);
        assert_eq!(BootTask::from(2), BootTask::RunSelfTest);
        assert_eq!(BootTask::from(999), BootTask::None); // Unknown values default to None
    }

    #[test]
    fn test_boot_task_repr() {
        assert_eq!(BootTask::None as u32, 0);
        assert_eq!(BootTask::UpdateFirmware as u32, 1);
        assert_eq!(BootTask::RunSelfTest as u32, 2);
    }

    #[test]
    fn test_backup_register_repr() {
        assert_eq!(BackupRegister::BootTask as usize, 0);
        assert_eq!(BackupRegister::BootCounter as usize, 1);
    }
}