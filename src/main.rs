#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _; // global logger
use panic_probe as _;

use embassy_executor::Spawner;

// Import hardware abstraction and application logic
use sensor_swarm::hw::BlackPillDevice;
use sensor_swarm::hw::traits::DeviceManagement;
use sensor_swarm::hw::blackpill_f401::usb::UsbManager;
use sensor_swarm::hw::blackpill_f401::usb_defmt_logger::process_usb_log_queue;
use sensor_swarm::app::SensorApp;
use sensor_swarm::usb_log;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    
    // Initialize device manager
    let mut device_manager = BlackPillDevice::new();
    
    // Initialize all hardware peripherals using hardware abstraction
    let (led, usb_manager) = match device_manager.init_peripherals(p).await {
        Ok((led, usb_manager)) => {
            info!("Hardware peripherals initialized successfully");
            
            // Initialize USB logging bridge to forward defmt logs to USB
            // Note: In a real implementation, we'd need to handle the static lifetime properly
            // For now, we'll demonstrate the concept
            usb_log!(info, "USB logging bridge is now active!");
            usb_log!(info, "All logs will be sent to both RTT and USB serial");
            
            (led, usb_manager)
        },
        Err(e) => {
            info!("Failed to initialize hardware peripherals: {}", e);
            panic!("Hardware initialization failed");
        }
    };
    
    // Spawn USB device task to handle USB communication and log forwarding
    spawner.spawn(usb_device_task(usb_manager)).unwrap();
    
    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);
    
    // Run the main application logic (hardware-agnostic)
    app.run().await;
}

#[embassy_executor::task]
async fn usb_device_task(mut usb_manager: UsbManager) {
    info!("Starting USB device task with log forwarding");
    loop {
        // Process any queued USB log messages
        process_usb_log_queue(&mut usb_manager).await;
        
        // Run the USB device state machine
        if let Err(e) = usb_manager.run_usb_task().await {
            info!("USB device task error: {}", e);
            // Wait a bit before retrying
            embassy_time::Timer::after_millis(100).await;
        }
        
        // Small delay to prevent busy waiting
        embassy_time::Timer::after_millis(10).await;
    }
}
