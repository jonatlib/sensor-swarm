/// Hardware-agnostic backup domain management
/// This module provides safe, reusable structures for managing tasks that need to be
/// performed after a device reset, leveraging Rust's type system to prevent common bugs.

use crate::hw::traits::BackupRegisters;
use crate::hw::{BackupRegister, BootTask};

/// A high-level handle for managing backup domain operations.
/// This struct provides a hardware-agnostic interface for backup register operations
/// while ensuring exclusive access through Rust's borrowing system.
pub struct BackupDomain<B>
where
    B: BackupRegisters,
{
    backup_registers: B,
}

impl<B> BackupDomain<B>
where
    B: BackupRegisters,
{
    /// Creates a new BackupDomain from an initialized backup registers implementation.
    /// 
    /// # Arguments
    /// * `backup_registers` - An implementation of the BackupRegisters trait
    /// 
    /// # Returns
    /// A new BackupDomain instance
    pub fn new(backup_registers: B) -> Self {
        Self { backup_registers }
    }

    /// Provides a specialized accessor for the boot task register.
    /// It takes a mutable reference to self to ensure exclusive access.
    /// 
    /// # Returns
    /// A BootTaskAccessor that provides safe access to boot task operations
    pub fn boot_task(&mut self) -> BootTaskAccessor<'_, B> {
        BootTaskAccessor { domain: self }
    }
}

/// A specialized accessor for reading and writing the `BootTask`.
/// This struct ensures that operations on this specific register are handled correctly
/// and prevents common firmware bugs through Rust's type system.
pub struct BootTaskAccessor<'a, B>
where
    B: BackupRegisters,
{
    domain: &'a mut BackupDomain<B>,
}

impl<'a, B> BootTaskAccessor<'a, B>
where
    B: BackupRegisters,
{
    /// Reads the boot task from the register AND immediately clears it.
    /// This atomic read-and-clear prevents the task from being executed more than once.
    /// 
    /// # Returns
    /// The BootTask that was stored in the register before clearing
    pub fn read_and_clear(&mut self) -> BootTask {
        let task_reg = BackupRegister::BootTask as usize;
        let raw_value = self.domain.backup_registers.read_register(task_reg);
        
        // Clear the register immediately after reading
        self.domain.backup_registers.write_register(task_reg, BootTask::None as u32);
        
        BootTask::from(raw_value)
    }

    /// Writes a new boot task to the register.
    /// Typically used before triggering a software reset.
    /// 
    /// # Arguments
    /// * `task` - The BootTask to store in the backup register
    pub fn write(&mut self, task: BootTask) {
        self.domain
            .backup_registers
            .write_register(BackupRegister::BootTask as usize, task as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation of BackupRegisters for testing
    struct MockBackupRegisters {
        registers: [u32; 20],
    }

    impl MockBackupRegisters {
        fn new() -> Self {
            Self {
                registers: [0; 20],
            }
        }
    }

    impl BackupRegisters for MockBackupRegisters {
        fn read_register(&self, index: usize) -> u32 {
            assert!(index < self.register_count());
            self.registers[index]
        }

        fn write_register(&mut self, index: usize, value: u32) {
            assert!(index < self.register_count());
            self.registers[index] = value;
        }

        fn register_count(&self) -> usize {
            20
        }
    }

    #[test]
    fn test_backup_domain_creation() {
        let mock_registers = MockBackupRegisters::new();
        let _domain = BackupDomain::new(mock_registers);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_boot_task_write_and_read() {
        let mock_registers = MockBackupRegisters::new();
        let mut domain = BackupDomain::new(mock_registers);
        
        // Write a task
        domain.boot_task().write(BootTask::UpdateFirmware);
        
        // Read and verify it was written correctly
        let task = domain.boot_task().read_and_clear();
        assert_eq!(task, BootTask::UpdateFirmware);
        
        // Verify the register was cleared after read
        let task_after_clear = domain.boot_task().read_and_clear();
        assert_eq!(task_after_clear, BootTask::None);
    }

    #[test]
    fn test_boot_task_read_and_clear_atomicity() {
        let mock_registers = MockBackupRegisters::new();
        let mut domain = BackupDomain::new(mock_registers);
        
        // Write a task
        domain.boot_task().write(BootTask::RunSelfTest);
        
        // Read and clear should return the task and clear it atomically
        let task = domain.boot_task().read_and_clear();
        assert_eq!(task, BootTask::RunSelfTest);
        
        // Subsequent reads should return None
        let task2 = domain.boot_task().read_and_clear();
        assert_eq!(task2, BootTask::None);
    }

    #[test]
    fn test_multiple_writes() {
        let mock_registers = MockBackupRegisters::new();
        let mut domain = BackupDomain::new(mock_registers);
        
        // Write different tasks and verify each one
        domain.boot_task().write(BootTask::UpdateFirmware);
        let task1 = domain.boot_task().read_and_clear();
        assert_eq!(task1, BootTask::UpdateFirmware);
        
        domain.boot_task().write(BootTask::RunSelfTest);
        let task2 = domain.boot_task().read_and_clear();
        assert_eq!(task2, BootTask::RunSelfTest);
        
        domain.boot_task().write(BootTask::None);
        let task3 = domain.boot_task().read_and_clear();
        assert_eq!(task3, BootTask::None);
    }
}