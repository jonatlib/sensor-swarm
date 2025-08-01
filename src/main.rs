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
use embassy_futures::join::join;
// Import hardware abstraction and application logic
use sensor_swarm::app::SensorApp;
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
    let (usb_wrapper, _remaining_peripherals) = device_manager
        .init_usb(remaining_peripherals)
        .await
        .expect("USB initialization failed");

    info!("Hardware peripherals initialized successfully");

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

    // Spawn USB tasks using the wrapper
    spawner.spawn(usb_device_task(usb_wrapper)).unwrap();

    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);

    // Run the main application logic (hardware-agnostic)
    app.run().await;
}

#[embassy_executor::task]
async fn usb_device_task(mut usb_wrapper: sensor_swarm::hw::blackpill_f401::usb::UsbWrapper) {
    info!("Starting USB device and CDC task");
    
    loop {
        // Process any queued USB log messages
        process_usb_log_queue(&mut usb_wrapper).await;
        
        // Handle CDC communication (this includes waiting for connection)
        let _ = usb_wrapper.handle_cdc_communication().await;
        
        // Small delay before reconnecting
        embassy_time::Timer::after_millis(100).await;
    }
}
