/// Enhanced GPIO implementation for STM32F401 Black Pill
/// Provides hardware-specific GPIO initialization functions that return Embassy GPIO types

use embassy_stm32::gpio::{Level, Output, Input, Pull, Speed, Pin, AnyPin};
use defmt::*;

/// GPIO initialization functions for STM32F401 Black Pill
/// These functions return configured Embassy GPIO types directly
pub struct BlackPillGpioInit;

impl BlackPillGpioInit {
    /// Create a new GPIO pin as output
    /// Returns Embassy Output type directly - no wrapper needed
    pub fn init_output(pin: impl Pin + 'static, initial_level: Level, speed: Speed) -> Output<'static, AnyPin> {
        info!("Initializing GPIO output pin with level: {:?}, speed: {:?}", initial_level, speed);
        Output::new(pin.degrade(), initial_level, speed)
    }

    /// Create a new GPIO pin as output with default settings (low level, low speed)
    pub fn init_output_default(pin: impl Pin + 'static) -> Output<'static, AnyPin> {
        Self::init_output(pin, Level::Low, Speed::Low)
    }

    /// Create a new GPIO pin as input
    /// Returns Embassy Input type directly - no wrapper needed
    pub fn init_input(pin: impl Pin + 'static, pull: Pull) -> Input<'static, AnyPin> {
        info!("Initializing GPIO input pin with pull: {:?}", pull);
        Input::new(pin.degrade(), pull)
    }

    /// Create a new GPIO pin as input with pull-up
    pub fn init_input_pullup(pin: impl Pin + 'static) -> Input<'static, AnyPin> {
        Self::init_input(pin, Pull::Up)
    }

    /// Create a new GPIO pin as input with pull-down
    pub fn init_input_pulldown(pin: impl Pin + 'static) -> Input<'static, AnyPin> {
        Self::init_input(pin, Pull::Down)
    }

    /// Create a new GPIO pin as input with no pull resistor
    pub fn init_input_floating(pin: impl Pin + 'static) -> Input<'static, AnyPin> {
        Self::init_input(pin, Pull::None)
    }
}

// GPIO input functionality is provided directly by Embassy Input type
// Use BlackPillGpioInit::init_input* functions to create Embassy Input types

/// GPIO manager for organizing multiple pins
/// Provides convenient access to commonly used pins on Black Pill
pub struct BlackPillGpioManager {
    // Common pins that might be used
    pins_initialized: bool,
}

impl BlackPillGpioManager {
    /// Create a new GPIO manager
    pub fn new() -> Self {
        Self {
            pins_initialized: false,
        }
    }

    /// Initialize the GPIO manager
    pub fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing GPIO manager...");
        
        // GPIO initialization is handled per-pin basis
        // This method can be used for any global GPIO setup if needed
        
        self.pins_initialized = true;
        info!("GPIO manager initialized successfully");
        Ok(())
    }

    /// Check if GPIO manager has been initialized
    pub fn is_initialized(&self) -> bool {
        self.pins_initialized
    }

    /// Get information about available GPIO pins on Black Pill
    pub fn get_pin_info(&self) -> GpioPinInfo {
        GpioPinInfo {
            total_pins: 32, // STM32F401CCU6 has 32 GPIO pins
            available_pins: &[
                "PA0", "PA1", "PA2", "PA3", "PA4", "PA5", "PA6", "PA7",
                "PA8", "PA9", "PA10", "PA11", "PA12", "PA13", "PA14", "PA15",
                "PB0", "PB1", "PB2", "PB3", "PB4", "PB5", "PB6", "PB7",
                "PB8", "PB9", "PB10", "PB12", "PB13", "PB14", "PB15",
                "PC13", "PC14", "PC15"
            ],
            special_pins: &[
                ("PC13", "Built-in LED (active low)"),
                ("PA11", "USB D-"),
                ("PA12", "USB D+"),
                ("PA13", "SWDIO"),
                ("PA14", "SWCLK"),
            ],
        }
    }
}

impl Default for BlackPillGpioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// GPIO pin information structure
#[derive(Debug, Clone)]
pub struct GpioPinInfo {
    pub total_pins: u32,
    pub available_pins: &'static [&'static str],
    pub special_pins: &'static [(&'static str, &'static str)],
}