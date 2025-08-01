#![no_std]
#![no_main]

use defmt::info;

use panic_probe as _;

// Logging
#[cfg(not(test))]
use defmt_rtt as _;
#[cfg(test)]
use defmt_semihosting as _;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
// Import hardware abstraction and application logic
use sensor_swarm::app::SensorApp;
use sensor_swarm::hw::blackpill_f401::usb::UsbManager;
use sensor_swarm::hw::blackpill_f401::usb_defmt_logger::process_usb_log_queue;
use sensor_swarm::hw::traits::{DeviceManagement, Led};
use sensor_swarm::hw::BlackPillDevice;
use sensor_swarm::usb_log;

#[cfg(not(test))]
#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Starting sensor swarm application");

    // Initialize device manager
    let mut device_manager = BlackPillDevice::new();

    // Get the device-specific configuration for embassy initialization
    let embassy_config = device_manager.init().expect("Device initialization failed");
    let p = embassy_stm32::init(embassy_config);

    // Initialize LED first for early debugging (hardware-agnostic)
    let (mut led, remaining_peripherals) = device_manager
        .init_led(p)
        .expect("LED initialization failed");

    // Blink LED once to indicate LED initialization complete
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(1000).await;
    led.off();
    embassy_time::Timer::after_millis(1000).await;

    // Initialize USB using hardware abstraction
    let (usb_manager, _remaining_peripherals) = device_manager
        .init_usb(remaining_peripherals)
        .await
        .expect("USB initialization failed");

    info!("Hardware peripherals initialized successfully");

    // Initialize USB logging bridge to forward defmt logs to USB
    // Note: In a real implementation, we'd need to handle the static lifetime properly
    // For now, we'll demonstrate the concept
    info!("USB logging bridge is now active!");
    info!("All logs will be sent to both RTT and USB serial");

    // Blink LED again to indicate all initialization complete
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    embassy_time::Timer::after_millis(1000).await;

    // Create a second USB manager instance for log processing (shares the same global USB components)
    let usb_log_manager = UsbManager::new();
    
    // Spawn USB device task to handle USB communication (runs continuously for enumeration)
    spawner.spawn(usb_device_task(usb_manager)).unwrap();
    
    // Spawn separate USB log processing task (with delays to not interfere with enumeration)
    spawner.spawn(usb_log_processing_task(usb_log_manager)).unwrap();

    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);

    // Run the main application logic (hardware-agnostic)
    app.run().await;
}

#[embassy_executor::task]
async fn usb_device_task(mut usb_manager: UsbManager) {
    info!("Starting USB device task - running continuously for proper enumeration");
    
    // Run the USB device state machine continuously
    // This is critical for USB enumeration - it must not be interrupted with delays
    loop {
        if let Err(e) = usb_manager.run_usb_task().await {
            info!("USB device task error: {}", e);
            // Only delay on error, not during normal operation
            embassy_time::Timer::after_millis(100).await;
        }
        // No delays during normal operation - USB needs continuous operation
    }
}

#[embassy_executor::task]
async fn usb_log_processing_task(mut usb_manager: UsbManager) {
    info!("Starting USB log processing task");
    loop {
        // Process any queued USB log messages
        // This runs separately from USB enumeration to avoid interference
        process_usb_log_queue(&mut usb_manager).await;
        
        // Small delay to prevent busy waiting and allow other tasks to run
        embassy_time::Timer::after_millis(50).await;
    }
}
