// Application logic module
// This module contains the main application logic for the sensor node
// All hardware interaction is done through traits to maintain hardware abstraction

use crate::hw::traits::{Led, DeviceManagement};
use embassy_time::Timer;
use defmt::info;

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
    pub async fn run(&mut self) {
        info!("Sensor swarm node starting with USB debugging...");
        info!("USB Serial debug interface is active!");
        
        let mut counter = 0;
        loop {
            // Heartbeat pattern using hardware-agnostic LED trait
            self.led.on();
            Timer::after_millis(500).await;
            self.led.off();
            Timer::after_millis(500).await;
            
            counter += 1;
            info!("Heartbeat #{}", counter);
            
            // Optional: Reboot to DFU bootloader after 5 seconds (5 heartbeats)
            if counter >= 5 {
                info!("Testing DFU bootloader reboot...");
                Timer::after_millis(1000).await; // Give time for the log message
                self.device_manager.reboot_to_bootloader();
            }
        }
    }
}