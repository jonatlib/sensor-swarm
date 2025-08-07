/// GPIO implementation for Raspberry Pi Pico (RP2040)
/// Provides GPIO pin management and configuration
use defmt::info;
use embassy_rp::gpio::{Input, Level, Output, Pull};

/// GPIO manager for Raspberry Pi Pico
/// Handles GPIO pin initialization and management
pub struct PiPicoGpioManager {
    // TODO: Add GPIO peripheral references if needed
}

impl PiPicoGpioManager {
    /// Create a new GPIO manager
    pub fn new() -> Self {
        info!("Initializing GPIO manager for RP2040");
        Self {}
    }
    
    /// Get information about available GPIO pins
    pub fn get_pin_info(&self, pin: u8) -> Option<GpioPinInfo> {
        if pin <= 28 {
            Some(GpioPinInfo {
                pin,
                name: match pin {
                    25 => "Built-in LED",
                    _ => "GPIO",
                },
                supports_pwm: true, // Most RP2040 pins support PWM
                supports_adc: matches!(pin, 26..=29), // ADC pins
            })
        } else {
            None
        }
    }
}

/// GPIO initialization helper for Raspberry Pi Pico
pub struct PiPicoGpioInit;

impl PiPicoGpioInit {
    /// Initialize a GPIO pin as output
    /// 
    /// # Arguments
    /// * `pin` - GPIO pin peripheral wrapped in Peri
    /// * `initial_level` - Initial output level
    /// 
    /// # Returns
    /// * `Output` - Configured output pin
    pub fn init_output(pin: embassy_rp::Peri<'static, impl embassy_rp::gpio::Pin>, initial_level: Level) -> Output<'static> {
        info!("Initializing GPIO pin as output");
        Output::new(pin, initial_level)
    }
    
    /// Initialize a GPIO pin as input
    /// 
    /// # Arguments
    /// * `pin` - GPIO pin peripheral wrapped in Peri
    /// * `pull` - Pull-up/pull-down configuration
    /// 
    /// # Returns
    /// * `Input` - Configured input pin
    pub fn init_input(pin: embassy_rp::Peri<'static, impl embassy_rp::gpio::Pin>, pull: Pull) -> Input<'static> {
        info!("Initializing GPIO pin as input");
        Input::new(pin, pull)
    }
}

/// GPIO pin information structure
#[derive(Debug, Clone)]
pub struct GpioPinInfo {
    pub pin: u8,
    pub name: &'static str,
    pub supports_pwm: bool,
    pub supports_adc: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test GPIO functionality
    /// 
    /// These tests verify the basic GPIO management functionality
    #[defmt_test::tests]
    mod gpio_tests {
        use super::*;
        
        /// Test GPIO manager creation
        #[test]
        fn test_gpio_manager_creation() {
            let gpio_manager = PiPicoGpioManager::new();
            
            // Test pin info for valid pins
            let pin_info = gpio_manager.get_pin_info(25);
            assert!(pin_info.is_some());
            
            let pin_info = pin_info.unwrap();
            assert_eq!(pin_info.pin, 25);
            assert_eq!(pin_info.name, "Built-in LED");
            assert!(pin_info.supports_pwm);
        }
        
        /// Test GPIO pin info for invalid pins
        #[test]
        fn test_invalid_gpio_pins() {
            let gpio_manager = PiPicoGpioManager::new();
            
            // Test invalid pin numbers
            assert!(gpio_manager.get_pin_info(30).is_none());
            assert!(gpio_manager.get_pin_info(255).is_none());
        }
        
        /// Test ADC pin detection
        #[test]
        fn test_adc_pin_detection() {
            let gpio_manager = PiPicoGpioManager::new();
            
            // Test ADC pins (26-29)
            for pin in 26..=29 {
                let pin_info = gpio_manager.get_pin_info(pin);
                assert!(pin_info.is_some());
                assert!(pin_info.unwrap().supports_adc);
            }
            
            // Test non-ADC pins
            let pin_info = gpio_manager.get_pin_info(25);
            assert!(pin_info.is_some());
            assert!(!pin_info.unwrap().supports_adc);
        }
    }
}