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
use sensor_swarm::backup_domain::BackupDomain;
use sensor_swarm::boot_task::execute_boot_task;
use sensor_swarm::hw::traits::{DeviceManagement, Led};
use sensor_swarm::hw::BlackPillDevice;
use sensor_swarm::usb::UsbCdcWrapper;
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

    // Initialize RTC and backup domain for boot task management FIRST (right after clock init)
    let (backup_registers, remaining_peripherals) = device_manager
        .init_rtc(p)
        .expect("RTC initialization failed");
    
    // Create backup domain for boot task management
    let mut backup_domain = BackupDomain::new(backup_registers);
    
    // Check for pending boot tasks and read them, then execute immediately
    let boot_task = backup_domain.boot_task().read_and_clear();
    info!("Boot task consumed: {:?}", boot_task);
    
    // Execute the boot task directly (not as a spawned task)
    execute_boot_task(boot_task);

    // Initialize LED first for early debugging (hardware-agnostic)
    let (mut led, remaining_peripherals) = device_manager
        .init_led(remaining_peripherals)
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
    let (usb_wrapper, remaining_peripherals) = device_manager
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

    // Create shared terminal using the UsbCdcWrapper directly
    let shared_terminal = create_shared_terminal(usb_wrapper);

    // Create a separate device manager instance for the command handler
    let command_device_manager = BlackPillDevice::new();
    
    // Spawn command handler task using the new Terminal-based approach
    spawner.spawn(command_handler_task(shared_terminal, command_device_manager)).unwrap();

    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);

    // Run the main application logic (hardware-agnostic)
    app.run().await;
}


#[embassy_executor::task]
async fn command_handler_task(terminal: sensor_swarm::terminal::SharedTerminal<UsbCdcWrapper>, device_manager: BlackPillDevice) {
    info!("Starting command handler task using Terminal-based approach");

    // Run the command handler - it will handle connection waiting internally
    match run_command_handler(terminal, device_manager).await {
        Ok(_) => {
            info!("Command handler completed successfully");
        }
        Err(e) => {
            info!("Command handler error: {}", e);
        }
    }
}
