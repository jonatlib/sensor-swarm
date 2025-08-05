/// STM32F401 Black Pill backup register implementation
/// Provides hardware-specific access to backup registers via RTC peripheral

use crate::hw::traits::BackupRegisters;
use embassy_stm32::rtc::Rtc;

/// STM32F401 implementation of backup registers using RTC peripheral
/// The STM32F401 has 20 backup registers (32-bit each) that retain their values
/// across system resets but are lost on power loss or backup domain reset
pub struct BlackPillBackupRegisters {
    rtc: Rtc,
}

impl BlackPillBackupRegisters {
    /// Create a new backup registers instance from an initialized RTC peripheral
    /// 
    /// # Arguments
    /// * `rtc` - Initialized RTC peripheral from embassy_stm32
    /// 
    /// # Returns
    /// A new BlackPillBackupRegisters instance
    pub fn new(rtc: Rtc) -> Self {
        Self { rtc }
    }

    /// Get a reference to the underlying RTC peripheral
    /// This can be useful for other RTC operations if needed
    pub fn rtc(&self) -> &Rtc {
        &self.rtc
    }

    /// Get a mutable reference to the underlying RTC peripheral
    /// This can be useful for other RTC operations if needed
    pub fn rtc_mut(&mut self) -> &mut Rtc {
        &mut self.rtc
    }
}

impl BackupRegisters for BlackPillBackupRegisters {
    /// Read a u32 value from the specified backup register index
    /// 
    /// # Arguments
    /// * `index` - The backup register index (0-19 for STM32F401)
    /// 
    /// # Returns
    /// The u32 value stored in the backup register
    /// 
    /// # Panics
    /// Panics if index >= register_count()
    fn read_register(&self, index: usize) -> u32 {
        assert!(index < self.register_count(), "Backup register index {} out of range", index);
        self.rtc.read_backup_register(index).unwrap_or(0)
    }

    /// Write a u32 value to the specified backup register index
    /// 
    /// # Arguments
    /// * `index` - The backup register index (0-19 for STM32F401)
    /// * `value` - The u32 value to write to the backup register
    /// 
    /// # Panics
    /// Panics if index >= register_count()
    fn write_register(&mut self, index: usize, value: u32) {
        assert!(index < self.register_count(), "Backup register index {} out of range", index);
        self.rtc.write_backup_register(index, value);
    }

    /// Get the number of available backup registers
    /// STM32F401 has 20 backup registers (0-19)
    /// 
    /// # Returns
    /// The number of available backup registers (20 for STM32F401)
    fn register_count(&self) -> usize {
        20
    }
}
