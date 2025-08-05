// Application logic module
// This module contains the main application logic for the sensor node
// All hardware interaction is done through traits to maintain hardware abstraction

use crate::hw::traits::{DeviceManagement, Led};
use crate::terminal_log;
use embassy_time::Timer;

/// Main application structure that holds the hardware abstractions
pub struct SensorApp<L, D>
where
    L: Led,
    D: DeviceManagement,
{
    led: L,
    device_manager: D,
}

impl<L, D> SensorApp<L, D>
where
    L: Led,
    D: DeviceManagement,
{
    /// Create a new sensor application instance
    pub fn new(led: L, device_manager: D) -> Self {
        Self {
            led,
            device_manager,
        }
    }

    /// Run the main application loop
    /// This is the core application logic that is hardware-agnostic
    pub async fn run(&mut self) -> ! {
        // Use terminal_log! for hardware-agnostic logging
        terminal_log!(info, "Sensor swarm node starting with USB debugging...");

        // Use terminal_log! for hardware-agnostic logging
        terminal_log!(info, "USB Serial debug interface is active!");
        terminal_log!(
            info,
            "Application started - logs will appear on both RTT and USB serial"
        );

        let mut counter = 0;
        loop {
            // Heartbeat pattern using hardware-agnostic LED trait
            self.led.on();
            Timer::after_millis(100).await;
            self.led.off();
            Timer::after_millis(100).await;

            counter += 1;

            // Use hardware-agnostic logging
            terminal_log!(info, "Heartbeat #{}", counter);

            // Optional: Reboot to DFU bootloader after 10 seconds (10 heartbeats)
            // if counter >= 10 {
            //     terminal_log!(warn, "Testing DFU bootloader reboot in 2 seconds...");
            //     Timer::after_millis(2000).await; // Give time for the log message to be sent
            //     self.device_manager.reboot_to_bootloader();
            // }
        }
    }
}
