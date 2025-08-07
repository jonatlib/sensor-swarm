/// Mock implementations for BlackPill F401 hardware for testing
/// This module provides hardware-agnostic mock implementations that can be used
/// in tests without requiring actual hardware peripherals.
use crate::hw::traits::{BackupRegisters, DeviceInfo, DeviceManagement, Led};

/// Mock LED implementation for testing
/// Provides stub implementations of all LED operations
pub struct MockLed;

impl Led for MockLed {
    fn on(&mut self) {}
    fn off(&mut self) {}
    fn toggle(&mut self) {}
    fn set_brightness(&mut self, _brightness: u8) {}
}

/// Mock backup registers implementation for testing
/// Provides stub implementations that simulate backup register behavior
pub struct MockBackupRegisters;

impl BackupRegisters for MockBackupRegisters {
    fn read_register(&self, _index: usize) -> u32 {
        0
    }

    fn write_register(&mut self, _index: usize, _value: u32) {}

    fn register_count(&self) -> usize {
        2
    }
}

/// Mock device implementation for testing
/// Provides a complete mock implementation of the DeviceManagement trait
/// that can be used in tests without requiring actual hardware
pub struct MockDevice;

impl<'d> DeviceManagement<'d> for MockDevice {
    type Led = MockLed;
    type UsbWrapper = ();
    type BackupRegisters = MockBackupRegisters;
    type Peripherals = ();
    type Config = ();

    fn new_with_peripherals(
        _peripherals: Self::Peripherals,
    ) -> Result<(Self::Config, Self), &'static str> {
        Ok(((), MockDevice))
    }

    fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            model: "Mock Device",
            board: "Test Board",
            flash_size: 512 * 1024,
            ram_size: 96 * 1024,
            system_clock_hz: 84_000_000,
            usb_clock_hz: 48_000_000,
            unique_id_hex: self.get_unique_id_hex(),
        }
    }

    fn soft_reset(&self) -> ! {
        loop {}
    }

    fn create_led(&mut self) -> Result<Self::Led, &'static str> {
        Ok(MockLed)
    }

    async fn create_usb(&mut self) -> Result<Self::UsbWrapper, &'static str> {
        Ok(())
    }

    fn create_rtc(&mut self) -> Result<Self::BackupRegisters, &'static str> {
        Ok(MockBackupRegisters)
    }

    fn get_backup_registers(&mut self) -> Option<&mut Self::BackupRegisters> {
        None
    }

    fn reboot(&self) -> ! {
        loop {}
    }

    fn disable_interrupts(&self) {}

    fn deinitialize_rtc(&self) {}

    fn deinitialize_clocks(&self) {}

    fn clear_pending_interrupts(&self) {}

    fn jump_to_dfu_bootloader(&self) -> ! {
        loop {}
    }

    /// Get the unique hardware ID as a byte array (mock implementation)
    /// Returns a mock unique identifier as raw bytes for testing
    fn get_unique_id_bytes(&self) -> [u8; 12] {
        // Mock UID for testing - represents a fake STM32 unique ID
        [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44,
        ]
    }

    /// Get the unique hardware ID as a hexadecimal string (mock implementation)
    /// Returns a mock unique identifier formatted as a hex string for testing
    fn get_unique_id_hex(&self) -> heapless::String<24> {
        heapless::String::try_from("123456789ABCDEF011223344")
            .unwrap_or_else(|_| heapless::String::new())
    }
}

/// Get a mock hardware device for testing
///
/// This function returns a mock device implementation that can be used
/// in tests without requiring actual hardware peripherals. The mock device
/// implements all the necessary traits but with stub implementations.
///
/// # Returns
/// A MockDevice instance that implements DeviceManagement trait
///
/// # Examples
/// ```
/// use sensor_swarm::testing::blackpill_f401::get_hw_mock;
/// use sensor_swarm::boot_task::execute_boot_task;
/// use sensor_swarm::hw::BootTask;
///
/// let device = get_hw_mock();
/// execute_boot_task(BootTask::None, &device);
/// ```
pub fn get_hw_mock() -> MockDevice {
    MockDevice
}
