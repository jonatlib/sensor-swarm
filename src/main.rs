#![no_std]
#![no_main]

use defmt::info;
use panic_probe as _;

use embassy_executor::Spawner;

// Import hardware abstraction and application logic
use sensor_swarm::hw::{BlackPillLed, BlackPillDevice};
use sensor_swarm::app::SensorApp;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    
    // Initialize device manager
    let device_manager = BlackPillDevice::new();
    
    // Initialize built-in LED using hardware abstraction (PC13 on STM32F401 Black Pill)
    let led = BlackPillLed::new(p.PC13);
    
    // Create the hardware-agnostic sensor application
    let mut app = SensorApp::new(led, device_manager);
    
    // Run the main application logic (hardware-agnostic)
    app.run().await;
}
