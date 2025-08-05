#![no_std]
#![no_main]

// Module declarations
#[cfg(feature = "blackpill-f401")]
pub mod app;
#[cfg(feature = "blackpill-f401")]
pub mod backup_domain;
#[cfg(feature = "blackpill-f401")]
pub mod boot_task;
#[cfg(feature = "blackpill-f401")]
pub mod commands;
#[cfg(feature = "blackpill-f401")]
pub mod hw;
#[cfg(feature = "blackpill-f401")]
pub mod logging;
pub mod radio;
#[cfg(feature = "blackpill-f401")]
pub mod sensors;
#[cfg(feature = "blackpill-f401")]
pub mod terminal;
#[cfg(feature = "blackpill-f401")]
pub mod usb;

#[cfg(feature = "defmt-test")]
#[defmt_test::tests]
mod tests {
    use defmt::assert;

    #[test]
    fn dummy_test() {
        assert!(true);
    }

    // Tests from radio module (not gated behind embedded feature)
    #[test]
    fn test_packet_control_flags() {
        use crate::radio::protocol::*;
        let mut control = PacketControl::new();

        // Test initial state
        defmt::assert!(!control.is_ack_request());
        defmt::assert!(!control.is_ack());
        defmt::assert!(!control.is_emergency());
        defmt::assert!(!control.is_retransmit());

        // Test setting flags
        control.set_ack_request(true);
        defmt::assert!(control.is_ack_request());

        control.set_ack_response(true);
        defmt::assert!(control.is_ack());

        control.set_emergency(true);
        defmt::assert!(control.is_emergency());

        control.set_retransmit(true);
        defmt::assert!(control.is_retransmit());

        // Test unsetting flags
        control.set_ack_request(false);
        defmt::assert!(!control.is_ack_request());
    }

    #[test]
    fn test_packet_creation() {
        use crate::radio::protocol::*;
        let payload = b"Hello, World!";
        let packet = Packet::new(0x1234, 0x5678, 42, payload);

        defmt::assert!(packet.header.sender_id == 0x1234);
        defmt::assert!(packet.header.target_id == 0x5678);
        defmt::assert!(packet.header.sequence_number == 42);
        defmt::assert!(packet.header.payload_len == payload.len() as u8);
        defmt::assert!(packet.payload_data() == payload);
    }

    #[test]
    fn test_packet_serialization_deserialization() {
        use crate::radio::protocol::*;
        let original_payload = b"Test data 123";
        let original_packet = Packet::new(0xABCD, 0xEF01, 999, original_payload);

        // Serialize to bytes
        let bytes = original_packet.to_bytes();

        // Deserialize back to packet
        let deserialized_packet = Packet::from_bytes(&bytes);

        // Verify all fields match
        defmt::assert!(deserialized_packet.header.sender_id == original_packet.header.sender_id);
        defmt::assert!(deserialized_packet.header.target_id == original_packet.header.target_id);
        defmt::assert!(
            deserialized_packet.header.sequence_number == original_packet.header.sequence_number
        );
        defmt::assert!(
            deserialized_packet.header.payload_len == original_packet.header.payload_len
        );
        defmt::assert!(deserialized_packet.payload_data() == original_packet.payload_data());
        defmt::assert!(deserialized_packet == original_packet);
    }

    // Tests from embedded modules
    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_boot_task_from_u32() {
        use crate::hw::{BootTask, BackupRegister};
        defmt::assert!(BootTask::from(0) == BootTask::None);
        defmt::assert!(BootTask::from(1) == BootTask::UpdateFirmware);
        defmt::assert!(BootTask::from(2) == BootTask::RunSelfTest);
        defmt::assert!(BootTask::from(3) == BootTask::DFUReboot);
        defmt::assert!(BootTask::from(999) == BootTask::None); // Unknown values default to None
    }

    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_boot_task_repr() {
        use crate::hw::{BootTask, BackupRegister};
        defmt::assert!(BootTask::None as u32 == 0);
        defmt::assert!(BootTask::UpdateFirmware as u32 == 1);
        defmt::assert!(BootTask::RunSelfTest as u32 == 2);
        defmt::assert!(BootTask::DFUReboot as u32 == 3);
    }

    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_backup_register_repr() {
        use crate::hw::{BootTask, BackupRegister};
        defmt::assert!(BackupRegister::BootTask as usize == 0);
        defmt::assert!(BackupRegister::BootCounter as usize == 1);
    }

    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_execute_boot_task_none() {
        use crate::hw::BootTask;
        use crate::hw::BlackPillDevice;
        // Test that None task executes without panic
        let device = BlackPillDevice::new();
        crate::boot_task::execute_boot_task(BootTask::None, &device);
        // Test passes if no panic occurs
    }

    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_execute_boot_task_update_firmware() {
        use crate::hw::BootTask;
        use crate::hw::BlackPillDevice;
        // Test that UpdateFirmware task executes without panic
        let device = BlackPillDevice::new();
        crate::boot_task::execute_boot_task(BootTask::UpdateFirmware, &device);
        // Test passes if no panic occurs
    }

    #[cfg(feature = "blackpill-f401")]
    #[test]
    fn test_execute_boot_task_run_self_test() {
        use crate::hw::BootTask;
        use crate::hw::BlackPillDevice;
        // Test that RunSelfTest task executes without panic
        let device = BlackPillDevice::new();
        crate::boot_task::execute_boot_task(BootTask::RunSelfTest, &device);
        // Test passes if no panic occurs
    }

    // Note: We cannot test execute_boot_task(BootTask::DFUReboot) because
    // it calls enter_dfu_mode() which never returns and would reset the system.
    // This functionality must be tested on actual hardware.
}

#[cfg(feature = "defmt-test")]
use defmt_semihosting as _;
#[cfg(feature = "defmt-test")]
use panic_probe as _;

#[cfg(all(feature = "defmt-test", feature = "blackpill-f401"))]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(all(feature = "defmt-test", not(feature = "blackpill-f401")))]
#[defmt::panic_handler]
fn panic() -> ! {
    loop {}
}
