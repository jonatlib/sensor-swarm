/// Device initialization and management for STM32F401 Black Pill
/// Provides hardware-specific device setup and configuration

use crate::hw::traits::DeviceManagement;
use embassy_stm32::{Config, rcc};
use defmt::*;

/// Device manager for STM32F401 Black Pill
/// Handles device initialization, clock configuration, and system management
pub struct BlackPillDevice {
    initialized: bool,
}

impl BlackPillDevice {
    /// Create a new device manager instance
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize the device with proper clock configuration
    /// This sets up the system clocks, HSE oscillator, and PLL
    pub fn init(&mut self) -> Result<Config, &'static str> {
        info!("Initializing STM32F401 Black Pill device...");

        let mut config = Config::default();
        
        // Configure HSE (High Speed External) oscillator - 25MHz crystal
        config.rcc.hse = Some(rcc::Hse {
            freq: embassy_stm32::time::Hertz(25_000_000),
            mode: rcc::HseMode::Oscillator,
        });
        
        // Use HSE as PLL source
        config.rcc.pll_src = rcc::PllSource::HSE;
        
        // Configure PLL for optimal performance
        config.rcc.pll = Some(rcc::Pll {
            prediv: rcc::PllPreDiv::DIV25,      // 25MHz / 25 = 1MHz
            mul: rcc::PllMul::MUL336,           // 1MHz * 336 = 336MHz
            divp: Some(rcc::PllPDiv::DIV4),     // 336MHz / 4 = 84MHz (System clock)
            divq: Some(rcc::PllQDiv::DIV7),     // 336MHz / 7 = 48MHz (USB clock)
            divr: None,
        });
        
        // Configure bus prescalers
        config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;      // AHB = 84MHz
        config.rcc.apb1_pre = rcc::APBPrescaler::DIV2;     // APB1 = 42MHz
        config.rcc.apb2_pre = rcc::APBPrescaler::DIV1;     // APB2 = 84MHz
        
        // Use PLL as system clock
        config.rcc.sys = rcc::Sysclk::PLL1_P;

        self.initialized = true;
        info!("Device initialization completed successfully");
        info!("System clock: 84MHz, APB1: 42MHz, APB2: 84MHz, USB: 48MHz");
        
        Ok(config)
    }

    /// Check if the device has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get device information
    pub fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            model: "STM32F401CCU6",
            board: "Black Pill",
            flash_size: 256 * 1024,  // 256KB
            ram_size: 64 * 1024,     // 64KB
            system_clock_hz: 84_000_000,
            usb_clock_hz: 48_000_000,
        }
    }

    /// Perform a soft reset of the device
    pub fn soft_reset(&self) -> ! {
        info!("Performing soft reset...");
        cortex_m::peripheral::SCB::sys_reset();
    }
}

impl DeviceManagement for BlackPillDevice {
    /// Timer peripheral type - using TIM2 as default timer
    type Timer = embassy_stm32::peripherals::TIM2;
    /// SPI peripheral type - using SPI1 as default SPI
    type Spi = embassy_stm32::peripherals::SPI1;
    
    /// Initialize a timer peripheral and return it pre-configured
    /// Returns TIM2 peripheral that can be used directly with Embassy timer functionality
    fn init_timer(&mut self) -> Result<Self::Timer, &'static str> {
        info!("Initializing TIM2 peripheral for timer functionality");
        
        // In a full implementation, this would take the TIM2 peripheral from embassy_stm32::init()
        // and configure it appropriately. For now, we return an error since we can't create
        // peripheral instances without the actual hardware initialization.
        
        warn!("Timer peripheral initialization is a stub - peripheral should be obtained from embassy_stm32::init()");
        Err("Timer peripheral initialization not fully implemented - use embassy_stm32::init() to get peripherals")
    }
    
    /// Initialize an SPI peripheral and return it pre-configured
    /// Returns SPI1 peripheral that can be used directly with Embassy SPI functionality
    fn init_spi(&mut self) -> Result<Self::Spi, &'static str> {
        info!("Initializing SPI1 peripheral for SPI functionality");
        
        // In a full implementation, this would take the SPI1 peripheral from embassy_stm32::init()
        // and configure it appropriately. For now, we return an error since we can't create
        // peripheral instances without the actual hardware initialization.
        
        warn!("SPI peripheral initialization is a stub - peripheral should be obtained from embassy_stm32::init()");
        Err("SPI peripheral initialization not fully implemented - use embassy_stm32::init() to get peripherals")
    }

    /// Reboot the device into DFU bootloader mode
    /// This triggers a jump to the STM32 built-in DFU bootloader
    fn reboot_to_bootloader(&self) -> ! {
        info!("Rebooting to DFU bootloader...");
        
        // For STM32F401, we need to:
        // 1. Set a magic value in RAM that the bootloader checks
        // 2. Trigger a system reset
        
        // Disable interrupts
        cortex_m::interrupt::disable();
        
        // Set the magic value in RAM (0x1FFF0000 is the bootloader address for STM32F401)
        // We'll use a different approach: set the stack pointer and jump directly
        unsafe {
            // STM32F401 system memory (bootloader) starts at 0x1FFF0000
            let bootloader_addr = 0x1FFF0000u32;
            
            // Read the stack pointer and reset vector from bootloader
            let stack_ptr = core::ptr::read_volatile(bootloader_addr as *const u32);
            let reset_vector = core::ptr::read_volatile((bootloader_addr + 4) as *const u32);
            
            // Set stack pointer
            cortex_m::register::msp::write(stack_ptr);
            
            // Jump to bootloader
            let bootloader_entry: extern "C" fn() -> ! = core::mem::transmute(reset_vector);
            bootloader_entry();
        }
    }
}

impl Default for BlackPillDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// Device information structure
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub model: &'static str,
    pub board: &'static str,
    pub flash_size: u32,
    pub ram_size: u32,
    pub system_clock_hz: u32,
    pub usb_clock_hz: u32,
}