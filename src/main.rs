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

// Unified imports using conditional type aliases
use sensor_swarm::hw::traits::{DeviceManagement, Led};
use sensor_swarm::hw::{CurrentDevice, CurrentLed, CurrentUsbWrapper, init_embassy};

// BlackPill-specific imports
use sensor_swarm::app::SensorApp;
use sensor_swarm::backup_domain::BackupDomain;
use sensor_swarm::boot_task::execute_boot_task;
use sensor_swarm::commands::run_command_handler;
use sensor_swarm::commands::Response;
use sensor_swarm::terminal::create_shared_terminal;

/// Initialize device manager and embassy framework (unified version)
///
/// Returns the initialized device manager
fn init_device_and_embassy() -> CurrentDevice {
    info!("Initializing device and embassy framework");

    // Get embassy peripherals using current device configuration
    let p = init_embassy();

    // Create device manager with peripherals using new safe API
    let (_embassy_config, device_manager) =
        CurrentDevice::new_with_peripherals(p).expect("Device initialization failed");

    // Note: We can't re-initialize embassy after it's already initialized
    // The embassy_config is returned for reference but embassy is already configured
    info!("Device manager created with embassy config");

    // Log device information
    let device_info = device_manager.get_device_info();
    
    // Only log device info with Response format for BlackPill (which has Response type)
    let response: Response = device_info.into();
    let mut response_str = heapless::String::<512>::new();
    let _ = core::fmt::write(&mut response_str, format_args!("{response}"));
    info!("{}", response_str.as_str());

    device_manager
}

/// Initialize RTC, backup domain and execute boot tasks (unified version)
/// Note: RTC and boot tasks are only available on BlackPill
fn init_rtc_and_boot_tasks(device_manager: &mut CurrentDevice) {
    info!("Initializing RTC and processing boot tasks");

    let backup_registers = device_manager
        .create_rtc()
        .expect("RTC initialization failed");

    let mut backup_domain = BackupDomain::new(backup_registers);
    let boot_task = backup_domain.boot_task().read_and_clear();
    info!("Boot task consumed: {:?}", boot_task);

    execute_boot_task(boot_task, device_manager);
}

/// Blink LED to indicate initialization step completion (unified version)
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

/// Initialize LED and provide early status indication (unified version)
async fn init_led_with_status(device_manager: &mut CurrentDevice) -> CurrentLed {
    info!("Initializing LED for status indication");

    let mut led = device_manager
        .create_led()
        .expect("LED initialization failed");

    blink_led_init_complete(&mut led).await;

    led
}

/// Initialize USB and create terminal interface (unified version)
/// Note: USB and terminal functionality is only available on BlackPill
#[cfg(feature = "blackpill-f401")]
async fn init_usb_and_terminal(
    device_manager: &mut CurrentDevice,
) -> sensor_swarm::terminal::SharedTerminal<CurrentUsbWrapper> {
    info!("Initializing USB and terminal interface");

    let usb_wrapper = device_manager
        .create_usb()
        .await
        .expect("USB initialization failed");

    info!("Hardware peripherals initialized successfully");

    create_shared_terminal(usb_wrapper)
}

/// Start the command handler task (unified version)
/// Note: Command handler is only available on BlackPill
#[cfg(feature = "blackpill-f401")]
fn start_command_handler(
    _spawner: &Spawner,
    _terminal: sensor_swarm::terminal::SharedTerminal<CurrentUsbWrapper>,
) {
    info!("Command handler temporarily disabled - needs architectural fix");

    // TODO: Fix command handler creation with new peripheral management
    // The new API doesn't allow creating multiple device managers since
    // peripherals are consumed by the first instance. We need to either:
    // 1. Share the device manager instance (requires lifetime management)
    // 2. Create a separate command handler that doesn't need device manager
    // 3. Redesign the command handler architecture

    // Temporarily commented out:
    // let command_device_manager = CurrentDevice::new();
    // spawner.spawn(command_handler_task(terminal, command_device_manager)).unwrap();
}

#[cfg(not(test))]
#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Starting sensor swarm application");

    // Initialize device and embassy framework
    let mut device_manager = init_device_and_embassy();

    // Initialize RTC and process boot tasks (BlackPill only)
    init_rtc_and_boot_tasks(&mut device_manager);

    // Initialize LED with status indication
    let mut led = init_led_with_status(&mut device_manager).await;

    // FIXME get rid of the separation acording to a board type
    // BlackPill-specific functionality
    #[cfg(feature = "blackpill-f401")]
    {
        // Initialize USB and terminal
        let terminal = init_usb_and_terminal(&mut device_manager).await;

        // Final status indication
        blink_led_all_complete(&mut led).await;

        // Start command handler
        start_command_handler(&spawner, terminal);

        // Create and run the main application
        let mut app = SensorApp::new(led, device_manager);
        app.run().await
    }

    // PiPico-specific functionality
    #[cfg(feature = "pipico")]
    {
        info!("PiPico initialization complete");

        // Simple LED blink loop for now
        loop {
            blink_led_all_complete(&mut led).await;
            embassy_time::Timer::after_millis(2000).await;
        }
    }
}

#[cfg(feature = "blackpill-f401")]
#[embassy_executor::task]
async fn command_handler_task(
    terminal: sensor_swarm::terminal::SharedTerminal<CurrentUsbWrapper>,
    device_manager: CurrentDevice,
) {
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
