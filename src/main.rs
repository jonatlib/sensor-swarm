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
use embassy_futures::join::{join, join3};
// Import hardware abstraction and application logic
use sensor_swarm::app::SensorApp;
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

    // Split USB wrapper to use trait-based architecture with spawned tasks
    let (usb_device, cdc_wrapper) = usb_wrapper.split_with_traits();

    // Spawn USB device task
    spawner.spawn(usb_device_task(usb_device)).unwrap();

    // Spawn USB commands task that handles CDC communication and logging
    spawner.spawn(usb_commands_task(cdc_wrapper)).unwrap();

    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);

    // Run the main application logic (hardware-agnostic)
    app.run().await;
}

#[embassy_executor::task]
async fn usb_device_task(
    mut usb_device: embassy_usb::UsbDevice<
        'static,
        embassy_stm32::usb_otg::Driver<'static, embassy_stm32::peripherals::USB_OTG_FS>,
    >,
) {
    info!("Starting USB device task - running continuously for proper enumeration");

    // Run the USB device state machine continuously
    // This is critical for USB enumeration - it must not be interrupted with delays
    usb_device.run().await;
}

#[embassy_executor::task]
async fn usb_commands_task(mut cdc_wrapper: sensor_swarm::hw::blackpill_f401::usb::CdcWrapper) {
    info!("Starting USB commands task for CDC communication and logging");

    loop {
        // Wait for USB connection
        cdc_wrapper.wait_connection().await;
        info!("USB CDC connected - starting command and logging handling");

        // Handle communication (includes logging queue processing and command handling)
        let _ = cdc_wrapper.handle_communication().await;

        info!("USB CDC disconnected - waiting for reconnection");
        // Small delay before trying to reconnect
        embassy_time::Timer::after_millis(100).await;
    }
}
