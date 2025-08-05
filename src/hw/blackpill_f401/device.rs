use crate::hw::blackpill_f401::backup_registers::BlackPillBackupRegisters;
use crate::hw::blackpill_f401::led::BlackPillLed;
use crate::hw::blackpill_f401::usb::UsbManager;
/// Device initialization and management for STM32F401 Black Pill
/// Provides hardware-specific device setup and configuration
use crate::hw::traits::{DeviceManagement, DeviceInfo, InitResult};
use defmt::{info, warn};
use embassy_stm32::Config;

/// Device manager for STM32F401 Black Pill
/// Handles device initialization, clock configuration, and system management
pub struct BlackPillDevice {
    initialized: bool,
}

impl BlackPillDevice {
    /// Create a new device manager instance
    pub fn new() -> Self {
        Self { initialized: false }
    }

}

impl DeviceManagement for BlackPillDevice {
    /// Timer peripheral type - using TIM2 as default timer
    type Timer = embassy_stm32::peripherals::TIM2;
    /// SPI peripheral type - using SPI1 as default SPI
    type Spi = embassy_stm32::peripherals::SPI1;
    /// LED type - using BlackPillLed for PC13
    type Led = BlackPillLed;
    /// USB Wrapper type - using UsbCdcWrapper for USB communication
    type UsbWrapper = crate::usb::UsbCdcWrapper;
    /// BackupRegisters type - using BlackPillBackupRegisters for RTC backup registers
    type BackupRegisters = BlackPillBackupRegisters;

    /// Initialize the device with proper clock configuration
    /// This sets up the system clocks, HSE oscillator, and PLL
    /// Based on working Embassy USB example configuration
    fn init(&mut self) -> Result<embassy_stm32::Config, &'static str> {
        let mut config = Config::default();
        {
            use embassy_stm32::rcc::*;
            // Configure HSE (High Speed External) oscillator - 25MHz crystal on Black Pill
            config.rcc.hse = Some(Hse {
                freq: embassy_stm32::time::Hertz(25_000_000),
                mode: HseMode::Oscillator,
            });

            // Use HSE as PLL source
            config.rcc.pll_src = PllSource::HSE;

            // Configure PLL for Black Pill with 25MHz HSE
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV25,  // 25MHz / 25 = 1MHz
                mul: PllMul::MUL336,       // 1MHz * 336 = 336MHz
                divp: Some(PllPDiv::DIV4), // 336MHz / 4 = 84MHz (System clock)
                divq: Some(PllQDiv::DIV7), // 336MHz / 7 = 48MHz (USB clock)
                divr: None,
            });

            // Configure bus prescalers for 84MHz system clock
            config.rcc.ahb_pre = AHBPrescaler::DIV1; // AHB = 84MHz
            config.rcc.apb1_pre = APBPrescaler::DIV2; // APB1 = 42MHz (max 42MHz)
            config.rcc.apb2_pre = APBPrescaler::DIV1; // APB2 = 84MHz (max 84MHz)

            // Use PLL as system clock
            config.rcc.sys = Sysclk::PLL1_P;
        }

        self.initialized = true;

        Ok(config)
    }

    /// Check if the device has been initialized
    fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get device information
    fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            model: "STM32F401CCU6",
            board: "Black Pill",
            flash_size: 256 * 1024,      // 256KB
            ram_size: 64 * 1024,         // 64KB
            system_clock_hz: 84_000_000, // Updated to match Black Pill 25MHz HSE configuration
            usb_clock_hz: 48_000_000,
        }
    }

    /// Perform a soft reset of the device
    fn soft_reset(&self) -> ! {
        info!("Performing soft reset...");
        cortex_m::peripheral::SCB::sys_reset();
    }

    /// Initialize LED peripheral separately for early debugging
    /// This method takes the full peripherals struct and extracts PC13 for LED initialization
    /// Returns initialized LED instance and remaining peripherals
    fn init_led(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Led> {
        // Extract PC13 for LED initialization using unsafe pointer operations
        // This is necessary because Rust's ownership system doesn't allow partial moves
        // from structs while returning the remaining struct
        let (pc13, remaining_peripherals) = unsafe {
            let mut p = core::mem::ManuallyDrop::new(peripherals);
            let pc13 = core::ptr::read(&p.PC13);

            // Reconstruct peripherals without PC13 by creating a new instance
            // Note: This is a workaround - in a real implementation, we'd need
            // a proper way to handle partial peripheral extraction
            let remaining = core::ptr::read(&*p);
            (pc13, remaining)
        };

        let led = BlackPillLed::new(pc13);

        Ok((led, remaining_peripherals))
    }

    /// Initialize USB peripheral from embassy_stm32::init output
    /// This method takes the peripherals struct and extracts what it needs for USB initialization
    /// Returns initialized USB wrapper instance and remaining peripherals
    async fn init_usb(
        &mut self,
        peripherals: embassy_stm32::Peripherals,
    ) -> InitResult<Self::UsbWrapper> {
        info!("Initializing BlackPill USB...");

        // Extract USB peripherals using unsafe pointer operations
        let (usb_otg_fs, pa12, pa11, remaining_peripherals) = unsafe {
            let mut p = core::mem::ManuallyDrop::new(peripherals);
            let usb_otg_fs = core::ptr::read(&p.USB_OTG_FS);
            let pa12 = core::ptr::read(&p.PA12);
            let pa11 = core::ptr::read(&p.PA11);

            // Reconstruct peripherals without the extracted ones
            let remaining = core::ptr::read(&*p);
            (usb_otg_fs, pa12, pa11, remaining)
        };

        // Initialize USB manager
        let mut usb_manager = UsbManager::new();

        // Initialize USB with the required peripherals (PA11=D-, PA12=D+)
        match usb_manager
            .init_with_peripheral(usb_otg_fs, pa12, pa11)
            .await
        {
            Ok(usb_wrapper) => {
                info!("USB wrapper initialized successfully");
                info!( "BlackPill USB peripherals initialized successfully");
                Ok((usb_wrapper, remaining_peripherals))
            }
            Err(e) => {
                warn!( "Failed to initialize USB wrapper: {}", e);
                Err("Failed to initialize USB wrapper")
            }
        }
    }

    /// Initialize a timer peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for timer initialization
    /// Returns initialized timer instance and remaining peripherals
    fn init_timer(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Timer> {
        info!( "Initializing TIM2 peripheral for timer functionality");

        // In a full implementation, this would extract the TIM2 peripheral from embassy_stm32::init()
        // and configure it appropriately. For now, we return an error since we can't create
        // peripheral instances without the actual hardware initialization.

        warn!( "Timer peripheral initialization is a stub - peripheral should be obtained from embassy_stm32::init()");
        Err("Timer peripheral initialization not fully implemented - use embassy_stm32::init() to get peripherals")
    }

    /// Initialize an SPI peripheral and return it pre-configured
    /// This method takes the peripherals struct and extracts what it needs for SPI initialization
    /// Returns initialized SPI instance and remaining peripherals
    fn init_spi(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::Spi> {
        info!( "Initializing SPI1 peripheral for SPI functionality");

        // In a full implementation, this would extract the SPI1 peripheral from embassy_stm32::init()
        // and configure it appropriately. For now, we return an error since we can't create
        // peripheral instances without the actual hardware initialization.

        warn!( "SPI peripheral initialization is a stub - peripheral should be obtained from embassy_stm32::init()");
        Err("SPI peripheral initialization not fully implemented - use embassy_stm32::init() to get peripherals")
    }

    /// Initialize RTC peripheral and return backup registers wrapper
    /// This method takes the peripherals struct and extracts what it needs for RTC initialization
    /// Returns initialized backup registers instance and remaining peripherals
    fn init_rtc(&mut self, peripherals: embassy_stm32::Peripherals) -> InitResult<Self::BackupRegisters> {
        info!( "Initializing RTC peripheral for backup registers functionality");

        // Extract RTC peripheral using unsafe pointer operations
        let (rtc_peripheral, remaining_peripherals) = unsafe {
            let mut p = core::mem::ManuallyDrop::new(peripherals);
            let rtc_peripheral = core::ptr::read(&p.RTC);

            // Reconstruct peripherals without the extracted RTC
            let remaining = core::ptr::read(&*p);
            (rtc_peripheral, remaining)
        };

        // Initialize RTC with default configuration
        // The LSE clock source should be configured at the system level in embassy_stm32::init()
        let rtc_config = embassy_stm32::rtc::RtcConfig::default();

        // Create RTC instance
        let rtc = embassy_stm32::rtc::Rtc::new(rtc_peripheral, rtc_config);

        // Create backup registers wrapper
        let backup_registers = BlackPillBackupRegisters::new(rtc);

        info!( "RTC and backup registers initialized successfully");
        Ok((backup_registers, remaining_peripherals))
    }

    /// Reboot the device normally
    /// This performs a standard system reset
    fn reboot(&self) -> ! {
        info!( "Performing normal system reboot...");
        self.soft_reset()
    }

    /// Disable all interrupts to prevent interference during DFU transition
    /// This should disable both cortex-m interrupts and any hardware-specific interrupts
    fn disable_interrupts(&self) {
        info!( "Disabling interrupts...");
        
        // Disable all interrupts using cortex-m
        cortex_m::interrupt::disable();
        
        // Additional STM32-specific interrupt disabling if needed
        unsafe {
            // Disable systick
            let syst = &*cortex_m::peripheral::SYST::PTR;
            syst.csr.write(0);
        }
    }

    /// De-initialize the RTC peripheral
    /// This should reset the RTC to its default state and disable RTC clocking
    fn deinitialize_rtc(&self) {
        info!( "De-initializing RTC...");
        
        // For STM32F401, we need to access RTC registers to properly de-initialize
        // This is hardware-specific implementation for BlackPill F401
        unsafe {
            // Access RTC registers through STM32F4xx peripheral access
            // Note: This is a simplified implementation - in a full implementation
            // we would need to properly handle RTC domain protection and clocking
            warn!( "RTC de-initialization - basic implementation for STM32F401");
            
            // TODO: Implement full RTC de-initialization:
            // - Disable RTC interrupts
            // - Reset RTC configuration registers  
            // - Disable RTC clock if possible
        }
    }

    /// De-initialize system clocks and prescalers
    /// This should reset the clock configuration to default state
    fn deinitialize_clocks(&self) {
        info!( "De-initializing clocks and prescalers...");
        
        // For STM32F401, reset clock configuration to default HSI state
        // This is hardware-specific implementation for BlackPill F401
        unsafe {
            // Access RCC (Reset and Clock Control) registers
            // Note: This is a simplified implementation - in a full implementation
            // we would need to properly sequence the clock changes
            warn!( "Clock de-initialization - basic implementation for STM32F401");
            
            // TODO: Implement full clock de-initialization:
            // - Reset PLL configuration
            // - Switch to HSI (internal oscillator)
            // - Reset prescalers to default values
            // - Disable external oscillators if used
        }
    }

    /// Clear any pending interrupts
    /// This should clear all pending interrupts in the NVIC and other interrupt controllers
    fn clear_pending_interrupts(&self) {
        info!( "Clearing pending interrupts...");
        
        unsafe {
            // Clear all pending interrupts in NVIC
            let nvic = &*cortex_m::peripheral::NVIC::PTR;
            
            // Clear pending interrupts for all interrupt lines
            // STM32F4xx has up to 82 interrupts, so we need to clear multiple registers
            for i in 0..3 {
                nvic.icpr[i].write(0xFFFFFFFF);
            }
        }
    }

    /// Jump to the DFU bootloader without resetting the device
    /// This transfers control directly to the STM32 system DFU bootloader
    /// Note: This function will not return as it transfers control to the bootloader
    fn jump_to_dfu_bootloader(&self) -> ! {
        info!( "Jumping to DFU bootloader...");

        // For STM32F401, jump directly to the system DFU bootloader
        unsafe {
            // STM32F401 system memory (bootloader) starts at 0x1FFF0000
            let bootloader_addr = 0x1FFF0000u32;

            // Read the stack pointer and reset vector from bootloader
            let stack_ptr = core::ptr::read_volatile(bootloader_addr as *const u32);
            let reset_vector = core::ptr::read_volatile((bootloader_addr + 4) as *const u32);

            info!( "Bootloader stack pointer: 0x{:08X}", stack_ptr);
            info!( "Bootloader entry point: 0x{:08X}", reset_vector);

            // Set stack pointer
            cortex_m::register::msp::write(stack_ptr);

            // Jump to bootloader - this will not return
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

