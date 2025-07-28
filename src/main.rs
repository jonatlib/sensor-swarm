#![no_std]
#![no_main]

use defmt::info;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;

// Import hardware abstraction
use sensor_swarm::hw::{DebugInterface, DeviceManagement, UsbManager};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    
    // Initialize USB Manager for debugging and device management
    let mut usb_manager = UsbManager::new();
    
    // Initialize USB peripheral with the required pins
    if let Err(e) = usb_manager.init_with_peripheral(p.USB_OTG_FS, p.PA12, p.PA11).await {
        // If USB initialization fails, we can't use USB logging
        // Fall back to basic operation without USB
        info!("USB initialization failed: {}", e);
    } else {
        // Initialize the debug interface
        if let Err(e) = usb_manager.init().await {
            info!("Debug interface initialization failed: {}", e);
        }
    }
    
    // Initialize built-in LED (PC13 on STM32F401 Black Pill)
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    
    info!("Sensor swarm node starting with USB debugging...");
    info!("USB Serial debug interface is active!");
    
    // Optional: Test DFU bootloader functionality after 5 seconds
    let mut counter = 0;
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
        
        counter += 1;
        info!("Heartbeat #{}", counter);
        
        // Optional: Reboot to DFU bootloader after 5 seconds (5 heartbeats)
        if counter >= 5 {
            info!("Testing DFU bootloader reboot...");
            Timer::after_millis(1000).await; // Give time for the log message
            usb_manager.reboot_to_bootloader();
        }
    }
}
