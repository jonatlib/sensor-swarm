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
// Embassy futures join removed - not needed with new architecture
// Import hardware abstraction and application logic
use sensor_swarm::app::SensorApp;
use sensor_swarm::hw::traits::{DeviceManagement, Led};
use sensor_swarm::hw::BlackPillDevice;
use sensor_swarm::hw::blackpill_f401::usb::UsbCdcWrapper;
use sensor_swarm::terminal::create_shared_terminal;
use sensor_swarm::commands::run_command_handler;

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

    // Split USB wrapper to get USB device and CDC class
    let (usb_device, cdc_class) = usb_wrapper.split();

    // Create UsbCdcWrapper from CDC class
    let usb_cdc_wrapper = UsbCdcWrapper::new(cdc_class);

    // Create shared terminal using the UsbCdcWrapper
    let shared_terminal = create_shared_terminal(usb_cdc_wrapper);

    // Spawn USB device task
    spawner.spawn(usb_device_task(usb_device)).unwrap();

    // Spawn command handler task using the new Terminal-based approach
    spawner.spawn(command_handler_task(shared_terminal)).unwrap();

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
async fn command_handler_task(terminal: sensor_swarm::terminal::SharedTerminal<UsbCdcWrapper>) {
    info!("Starting command handler task using Terminal-based approach");

    // Run the command handler - it will handle connection waiting internally
    match run_command_handler(terminal).await {
        Ok(_) => {
            info!("Command handler completed successfully");
        }
        Err(e) => {
            info!("Command handler error: {}", e);
        }
    }
}
