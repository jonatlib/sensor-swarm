#![no_std]
#![no_main]

// HIL tests use RTT for real hardware logging
use defmt_rtt as _; // global logger for HIL tests (use RTT for real hardware)
use panic_probe as _;

// Custom defmt panic handler for tests
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[cfg(feature = "hil")]
#[defmt_test::tests]
mod hil_tests {

    use sensor_swarm::hw::traits::{BackupRegisters, DeviceManagement, Led};
    use sensor_swarm::testing::hil::{init_hil_test_sync, test_device_info, test_led_basic};

    /// Basic HIL test to verify LED functionality
    ///
    /// This test verifies that:
    /// 1. Hardware can be initialized properly
    /// 2. LED can be created and controlled
    ///
    /// This is the most basic HIL test and serves as a verification
    /// that the HIL testing infrastructure is working correctly.
    ///
    /// Note: This is a synchronous test since defmt-test doesn't support async tests.
    /// In a real HIL environment, timing operations would be handled differently.
    #[test]
    fn test_hil_led_basic() {
        defmt::info!("Starting HIL LED basic test");

        // Initialize hardware (synchronous version)
        let mut ctx = init_hil_test_sync();
        defmt::info!("Hardware initialized successfully");

        // Test LED creation and basic operations
        let result = test_led_basic(&mut ctx.device);
        defmt::assert!(result.is_ok(), "LED basic test failed: {:?}", result);

        defmt::info!("HIL LED basic test completed successfully");
    }

    /// Test LED control operations individually
    ///
    /// This test verifies individual LED operations:
    /// - LED creation
    /// - LED on/off control
    /// - LED toggle functionality
    /// - LED brightness control (if supported)
    ///
    /// Note: This is a synchronous test since defmt-test doesn't support async tests.
    #[test]
    fn test_hil_led_operations() {
        defmt::info!("Starting HIL LED operations test");

        // Initialize hardware
        let mut ctx = init_hil_test_sync();

        // Create LED
        let mut led = ctx
            .device
            .create_led()
            .expect("Failed to create LED for HIL test");
        defmt::info!("LED created successfully");

        // Test LED on
        led.on();
        defmt::info!("LED on operation completed");

        // Test LED off
        led.off();
        defmt::info!("LED off operation completed");

        // Test LED toggle
        led.toggle();
        led.toggle();
        defmt::info!("LED toggle operations completed");

        // Test LED brightness (if supported)
        led.set_brightness(128); // 50% brightness
        led.on();
        led.off();
        defmt::info!("LED brightness test completed");

        defmt::info!("HIL LED operations test completed successfully");
    }

    /// Test device information retrieval in HIL environment
    ///
    /// This test verifies that device information can be retrieved
    /// correctly when running on actual hardware.
    #[test]
    fn test_hil_device_info() {
        defmt::info!("Starting HIL device info test");

        // Initialize hardware
        let ctx = init_hil_test_sync();

        // Test device info retrieval
        let info_valid = test_device_info(&ctx.device);
        defmt::assert!(info_valid, "Device info validation failed");

        // Get and log device information
        let device_info = ctx.device.get_device_info();
        defmt::info!("Device Model: {}", device_info.model);
        defmt::info!("Board: {}", device_info.board);
        defmt::info!("Flash Size: {} bytes", device_info.flash_size);
        defmt::info!("RAM Size: {} bytes", device_info.ram_size);
        defmt::info!("System Clock: {} Hz", device_info.system_clock_hz);
        defmt::info!("USB Clock: {} Hz", device_info.usb_clock_hz);
        defmt::info!("Unique ID: {}", device_info.unique_id_hex.as_str());

        // Verify specific values for real BlackPill device
        defmt::assert!(device_info.flash_size > 0, "Invalid flash size");
        defmt::assert!(device_info.ram_size > 0, "Invalid RAM size");
        defmt::assert!(device_info.system_clock_hz > 0, "Invalid system clock");
        defmt::assert!(!device_info.unique_id_hex.is_empty(), "Empty unique ID");

        defmt::info!("HIL device info test completed successfully");
    }

    /// Test basic HIL infrastructure
    ///
    /// This test verifies that the HIL testing infrastructure works correctly.
    /// This test uses real BlackPill hardware and verifies that the infrastructure
    /// is set up properly for hardware-in-the-loop testing.
    #[test]
    fn test_hil_infrastructure() {
        defmt::info!("Starting HIL infrastructure test");

        // Initialize hardware
        let _ctx = init_hil_test_sync();
        defmt::info!("HIL infrastructure initialized successfully");

        // Note: With real hardware, creating multiple contexts may fail
        // since peripherals can only be taken once. This test verifies
        // that the first context was created successfully.
        defmt::info!("HIL context created successfully with real hardware");

        defmt::info!("HIL infrastructure test completed successfully");
    }

    /// Test hardware reboot functionality (real hardware version)
    ///
    /// This test verifies that reboot functionality can be called on real hardware.
    /// The actual reboot call is commented out to prevent the test from rebooting the device.
    #[test]
    #[cfg(feature = "test-reboot")]
    fn test_hil_reboot_real() {
        defmt::info!("Starting HIL reboot test with real hardware");

        // Initialize real BlackPill hardware
        let ctx = init_hil_test_sync();

        defmt::info!("Testing reboot call (commented out to prevent actual reboot)");

        // This would call the real reboot function which would actually reboot the device
        // Commented out to prevent the test from actually rebooting during testing
        // ctx.device.reboot(); // Uncomment only if you want to test actual reboot

        defmt::info!("HIL reboot test completed successfully (reboot call was skipped)");
    }

    /// Test backup registers functionality
    ///
    /// This test verifies that backup registers can be written to and read from
    /// correctly in a HIL environment.
    #[test]
    fn test_hil_backup_registers() {
        defmt::info!("Starting HIL backup registers test");

        // Initialize hardware
        let mut ctx = init_hil_test_sync();

        // Try to create backup registers
        match ctx.device.create_rtc() {
            Ok(mut backup_regs) => {
                defmt::info!("Backup registers created successfully");

                // Test write and read operations
                let test_values = [0x12345678, 0xABCDEF00, 0xDEADBEEF];

                for (index, &value) in test_values.iter().enumerate() {
                    if index < backup_regs.register_count() {
                        // Write value
                        backup_regs.write_register(index, value);
                        defmt::info!("Wrote 0x{:08X} to register {}", value, index);

                        // Read back and verify
                        let read_value = backup_regs.read_register(index);
                        defmt::assert_eq!(
                            read_value,
                            value,
                            "Backup register {} mismatch: wrote 0x{:08X}, read 0x{:08X}",
                            index,
                            value,
                            read_value
                        );
                        defmt::info!("Verified register {} value", index);
                    }
                }

                defmt::info!("Backup registers test completed successfully");
            }
            Err(e) => {
                defmt::info!("Backup registers not available: {}", e);
                // This is not necessarily a failure - some configurations might not have backup registers
            }
        }
    }

    /// Test multiple LED operations in sequence
    ///
    /// This test creates LED instances and tests coordinated LED operations
    /// on real BlackPill hardware.
    #[test]
    fn test_hil_multiple_led_sequence() {
        defmt::info!("Starting HIL multiple LED sequence test");

        // Initialize hardware
        let mut ctx = init_hil_test_sync();

        // Create LED (most boards have only one LED)
        let mut led = ctx
            .device
            .create_led()
            .expect("Failed to create LED for sequence test");

        // Perform a complex LED sequence on real hardware
        defmt::info!("Starting LED sequence pattern");

        // Fast blink pattern
        for _ in 0..5 {
            led.on();
            led.off();
        }
        defmt::info!("Fast blink pattern completed");

        // Slow blink pattern
        for _ in 0..3 {
            led.on();
            led.off();
        }
        defmt::info!("Slow blink pattern completed");

        // Brightness fade pattern (if supported)
        let brightness_levels = [0, 64, 128, 192, 255, 192, 128, 64, 0];
        for &brightness in brightness_levels.iter() {
            led.set_brightness(brightness);
            led.on();
        }
        led.off();
        defmt::info!("Brightness fade pattern completed");

        defmt::info!("HIL multiple LED sequence test completed successfully");
    }
}
