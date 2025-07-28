/// GPIO and LED implementations for STM32F401 Black Pill
/// Provides hardware-specific implementations of GPIO and LED traits

use crate::hw::traits::Led;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::PC13;

/// Built-in LED implementation for STM32F401 Black Pill
/// The built-in LED is connected to PC13 and is active low
pub struct BlackPillLed {
    pin: Output<'static, PC13>,
}

impl BlackPillLed {
    /// Create a new LED instance using PC13 pin
    pub fn new(pc13_pin: PC13) -> Self {
        // LED is active low, so start with high level (LED off)
        let pin = Output::new(pc13_pin, Level::High, Speed::Low);
        Self { pin }
    }
}

impl Led for BlackPillLed {
    fn on(&mut self) {
        // LED is active low, so set pin low to turn on
        self.pin.set_low();
    }
    
    fn off(&mut self) {
        // LED is active low, so set pin high to turn off
        self.pin.set_high();
    }
    
    fn toggle(&mut self) {
        self.pin.toggle();
    }
}

/// Simple mock device management for testing (without USB)
pub struct MockDeviceManager;

impl MockDeviceManager {
    pub fn new() -> Self {
        Self
    }
}

impl crate::hw::traits::DeviceManagement for MockDeviceManager {
    fn reboot_to_bootloader(&self) -> ! {
        use defmt::info;
        info!("Mock: Would reboot to bootloader here");
        // In a real implementation, this would reboot to bootloader
        // For testing, we'll just loop forever
        loop {
            cortex_m::asm::wfi();
        }
    }
}