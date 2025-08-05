#![no_std]
#![no_main]

use defmt::info;

use panic_probe as _;

// Logging
#[cfg(all(not(test), not(feature = "defmt-test")))]
use defmt_rtt as _;
#[cfg(any(test, feature = "defmt-test"))]
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

/// Initialize device manager and embassy framework
/// 
/// Returns the initialized device manager and embassy peripherals
fn init_device_and_embassy() -> (BlackPillDevice, embassy_stm32::Peripherals) {
    info!("Initializing device and embassy framework");
    
    let mut device_manager = BlackPillDevice::new();
    let embassy_config = device_manager.init().expect("Device initialization failed");
    let p = embassy_stm32::init(embassy_config);
    
    (device_manager, p)
}

/// Initialize RTC, backup domain and execute boot tasks
/// 
/// Returns the backup domain and remaining peripherals after RTC initialization
fn init_rtc_and_boot_tasks(
    device_manager: &mut BlackPillDevice,
    peripherals: embassy_stm32::Peripherals,
) -> embassy_stm32::Peripherals {
    info!("Initializing RTC and processing boot tasks");
    
    let (backup_registers, remaining_peripherals) = device_manager
        .init_rtc(peripherals)
        .expect("RTC initialization failed");
    
    let mut backup_domain = BackupDomain::new(backup_registers);
    let boot_task = backup_domain.boot_task().read_and_clear();
    info!("Boot task consumed: {:?}", boot_task);
    
    execute_boot_task(boot_task, device_manager);
    
    remaining_peripherals
}

/// Blink LED to indicate initialization step completion
async fn blink_led_init_complete(led: &mut impl Led) {
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(1000).await;
    led.off();
    embassy_time::Timer::after_millis(1000).await;
}

/// Blink LED to indicate all initialization is complete
async fn blink_led_all_complete(led: &mut impl Led) {
    for _ in 0..3 {
        led.on();
        embassy_time::Timer::after_millis(200).await;
        led.off();
        embassy_time::Timer::after_millis(200).await;
    }
    embassy_time::Timer::after_millis(1000).await;
}

/// Initialize LED and provide early status indication
async fn init_led_with_status(
    device_manager: &mut BlackPillDevice,
    peripherals: embassy_stm32::Peripherals,
) -> (impl Led, embassy_stm32::Peripherals) {
    info!("Initializing LED for status indication");
    
    let (mut led, remaining_peripherals) = device_manager
        .init_led(peripherals)
        .expect("LED initialization failed");

    blink_led_init_complete(&mut led).await;
    
    (led, remaining_peripherals)
}

/// Initialize USB and create terminal interface
async fn init_usb_and_terminal(
    device_manager: &mut BlackPillDevice,
    peripherals: embassy_stm32::Peripherals,
) -> (sensor_swarm::terminal::SharedTerminal<UsbCdcWrapper>, embassy_stm32::Peripherals) {
    info!("Initializing USB and terminal interface");
    
    let (usb_wrapper, remaining_peripherals) = device_manager
        .init_usb(peripherals)
        .await
        .expect("USB initialization failed");

    info!("Hardware peripherals initialized successfully");
    
    let shared_terminal = create_shared_terminal(usb_wrapper);
    
    (shared_terminal, remaining_peripherals)
}

/// Start the command handler task
fn start_command_handler(
    spawner: &Spawner,
    terminal: sensor_swarm::terminal::SharedTerminal<UsbCdcWrapper>,
) {
    info!("Starting command handler task");
    
    // TODO: Consider sharing the device manager instance instead of creating a new one
    // This could lead to resource conflicts or inefficient resource usage
    let command_device_manager = BlackPillDevice::new();
    spawner.spawn(command_handler_task(terminal, command_device_manager)).unwrap();
}

#[cfg(not(test))]
#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Starting sensor swarm application");

    // Initialize device and embassy framework
    let (mut device_manager, peripherals) = init_device_and_embassy();
    
    // Initialize RTC and process boot tasks
    let peripherals = init_rtc_and_boot_tasks(&mut device_manager, peripherals);
    
    // Initialize LED with status indication
    let (mut led, peripherals) = init_led_with_status(&mut device_manager, peripherals).await;
    
    // Initialize USB and terminal
    let (terminal, _remaining_peripherals) = init_usb_and_terminal(&mut device_manager, peripherals).await;
    
    // Final status indication
    blink_led_all_complete(&mut led).await;
    
    // Start command handler
    start_command_handler(&spawner, terminal);
    
    // Create and run the main application
    let mut app = SensorApp::new(led, device_manager);
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
