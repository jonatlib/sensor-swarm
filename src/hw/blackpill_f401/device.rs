use crate::hw::blackpill_f401::backup_registers::BlackPillBackupRegisters;
use crate::hw::blackpill_f401::led::BlackPillLed;
use crate::hw::blackpill_f401::usb::UsbManager;
/// Device initialization and management for STM32F401 Black Pill
/// Provides hardware-specific device setup and configuration
use crate::hw::traits::{DeviceInfo, DeviceManagement};
use defmt::{info, warn};

/// Device manager for STM32F401 Black Pill
/// Handles device initialization, clock configuration, and system management
/// Stores peripherals individually to enable safe peripheral creation with lifetimes
pub struct BlackPillDevice {
    // Store individual peripherals as Options to allow safe extraction
    pc13: Option<embassy_stm32::peripherals::PC13>,
    usb_otg_fs: Option<embassy_stm32::peripherals::USB_OTG_FS>,
    pa12: Option<embassy_stm32::peripherals::PA12>,
    pa11: Option<embassy_stm32::peripherals::PA11>,
    rtc: Option<embassy_stm32::peripherals::RTC>,
    backup_registers: Option<BlackPillBackupRegisters>,
}

impl BlackPillDevice {
    /// Create a new device manager instance with peripherals stored internally
    /// This replaces the old unsafe peripheral-passing pattern
    fn new_internal(peripherals: embassy_stm32::Peripherals) -> Self {
        Self {
            pc13: Some(peripherals.PC13),
            usb_otg_fs: Some(peripherals.USB_OTG_FS),
            pa12: Some(peripherals.PA12),
            pa11: Some(peripherals.PA11),
            rtc: Some(peripherals.RTC),
            backup_registers: None,
        }
    }

    /// Get the Embassy configuration for STM32F401 Black Pill
    /// This is now a static method that doesn't require a device instance
    pub fn get_embassy_config() -> embassy_stm32::Config {
        let mut config = embassy_stm32::Config::default();
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
        config
    }
}

impl<'d> DeviceManagement<'d> for BlackPillDevice {
    /// LED type - using BlackPillLed for PC13
    type Led = BlackPillLed;
    /// USB Wrapper type - using UsbCdcWrapper for USB communication
    type UsbWrapper = crate::usb::UsbCdcWrapper;
    /// BackupRegisters type - using BlackPillBackupRegisters for RTC backup registers
    type BackupRegisters = BlackPillBackupRegisters;

    /// Create a new device manager instance with peripherals stored internally
    /// This static method returns the Embassy configuration and creates the device manager
    /// with the peripherals stored internally, eliminating unsafe pointer operations
    fn new_with_peripherals(
        peripherals: embassy_stm32::Peripherals,
    ) -> Result<(embassy_stm32::Config, Self), &'static str> {
        let config = Self::get_embassy_config();
        let device = Self::new_internal(peripherals);
        Ok((config, device))
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
            unique_id_hex: self.get_unique_id_hex(),
        }
    }

    /// Perform a soft reset of the device
    fn soft_reset(&self) -> ! {
        info!("Performing soft reset...");
        cortex_m::peripheral::SCB::sys_reset();
    }

    /// Create LED peripheral from stored peripherals for early debugging
    /// This method safely extracts PC13 from the internally stored peripherals
    /// The LED is bound to the device manager's lifetime, eliminating unsafe operations
    fn create_led(&'d mut self) -> Result<Self::Led, &'static str> {
        if let Some(pc13) = self.pc13.take() {
            let led = BlackPillLed::new(pc13);
            Ok(led)
        } else {
            Err("PC13 peripheral has already been consumed or not initialized")
        }
    }

    /// Create USB peripheral from stored peripherals
    /// This method safely extracts USB peripherals from the internally stored peripherals
    /// The USB wrapper is bound to the device manager's lifetime, eliminating unsafe operations
    async fn create_usb(&'d mut self) -> Result<Self::UsbWrapper, &'static str> {
        info!("Initializing BlackPill USB...");

        let usb_otg_fs = self
            .usb_otg_fs
            .take()
            .ok_or("USB_OTG_FS peripheral has already been consumed or not initialized")?;
        let pa12 = self
            .pa12
            .take()
            .ok_or("PA12 peripheral has already been consumed or not initialized")?;
        let pa11 = self
            .pa11
            .take()
            .ok_or("PA11 peripheral has already been consumed or not initialized")?;

        // Initialize USB manager
        let mut usb_manager = UsbManager::new();

        // Initialize USB with the required peripherals (PA11=D-, PA12=D+)
        match usb_manager
            .init_with_peripheral(usb_otg_fs, pa12, pa11)
            .await
        {
            Ok(usb_wrapper) => {
                info!("USB wrapper initialized successfully");
                info!("BlackPill USB peripherals initialized successfully");
                Ok(usb_wrapper)
            }
            Err(e) => {
                warn!("Failed to initialize USB wrapper: {}", e);
                Err("Failed to initialize USB wrapper")
            }
        }
    }

    /// Create RTC peripheral and backup registers from stored peripherals
    /// This method safely extracts RTC peripheral from the internally stored peripherals
    /// The backup registers are bound to the device manager's lifetime, eliminating unsafe operations
    fn create_rtc(&'d mut self) -> Result<Self::BackupRegisters, &'static str> {
        info!("Initializing RTC peripheral for backup registers functionality");

        let rtc_peripheral = self
            .rtc
            .take()
            .ok_or("RTC peripheral has already been consumed or not initialized")?;

        // Initialize RTC with default configuration
        // The LSE clock source should be configured at the system level in embassy_stm32::init()
        let rtc_config = embassy_stm32::rtc::RtcConfig::default();

        // Create RTC instance
        let rtc = embassy_stm32::rtc::Rtc::new(rtc_peripheral, rtc_config);

        // Create backup registers wrapper
        let backup_registers = BlackPillBackupRegisters::new(rtc);

        info!("RTC and backup registers initialized successfully");
        Ok(backup_registers)
    }

    /// Get access to backup registers for boot task management
    /// This method provides access to backup registers that have been created via create_rtc
    /// Returns None if backup registers haven't been created yet
    fn get_backup_registers(&mut self) -> Option<&mut Self::BackupRegisters> {
        self.backup_registers.as_mut()
    }

    /// Reboot the device normally
    /// This performs a standard system reset
    fn reboot(&self) -> ! {
        info!("Performing normal system reboot...");
        self.soft_reset()
    }

    /// Disable all interrupts to prevent interference during DFU transition
    /// This should disable both cortex-m interrupts and any hardware-specific interrupts
    fn disable_interrupts(&self) {
        info!("Disabling interrupts...");

        // Disable all interrupts using cortex-m
        cortex_m::interrupt::disable();

        // TODO: Add comprehensive interrupt disabling for production safety
        // Consider disabling all peripheral interrupts, not just SysTick
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
        info!("De-initializing RTC...");

        // For STM32F401, we need to access RTC registers to properly de-initialize
        // This is hardware-specific implementation for BlackPill F401
        unsafe {
            // Access RTC registers through STM32F4xx peripheral access
            // Note: This is a simplified implementation - in a full implementation
            // we would need to properly handle RTC domain protection and clocking
            warn!("RTC de-initialization - basic implementation for STM32F401");

            // TODO: Implement full RTC de-initialization:
            // - Disable RTC interrupts
            // - Reset RTC configuration registers
            // - Disable RTC clock if possible
        }
    }

    /// De-initialize system clocks and prescalers
    /// This should reset the clock configuration to default state
    fn deinitialize_clocks(&self) {
        info!("De-initializing clocks and prescalers...");

        // For STM32F401, reset clock configuration to default HSI state
        // This is hardware-specific implementation for BlackPill F401
        // TODO: Replace unsafe register access with safe HAL abstractions
        // This unsafe code should be replaced with proper embassy-stm32 APIs
        unsafe {
            // Access RCC (Reset and Clock Control) registers
            // Note: This is a simplified implementation - in a full implementation
            // we would need to properly sequence the clock changes
            warn!("Clock de-initialization - basic implementation for STM32F401");

            // TODO: Implement full clock de-initialization:
            // - Reset PLL configuration
            // - Switch to HSI (internal oscillator)
            // - Reset prescalers to default values
            // - Disable external oscillators if used
            // - Add proper error handling and timeout checks
        }
    }

    /// Clear any pending interrupts
    /// This should clear all pending interrupts in the NVIC and other interrupt controllers
    fn clear_pending_interrupts(&self) {
        info!("Clearing pending interrupts...");

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
        info!("Jumping to DFU bootloader...");

        // TODO: Add production safety checks for DFU bootloader jump
        // - Validate bootloader address and vectors before jumping
        // - Add timeout for bootloader detection
        // - Implement fallback mechanism if bootloader is corrupted
        // - Consider adding signature verification for security
        // For STM32F401, jump directly to the system DFU bootloader
        unsafe {
            // STM32F401 system memory (bootloader) starts at 0x1FFF0000
            let bootloader_addr = 0x1FFF0000u32;

            // TODO: Add validation of bootloader presence and integrity
            // Read the stack pointer and reset vector from bootloader
            let stack_ptr = core::ptr::read_volatile(bootloader_addr as *const u32);
            let reset_vector = core::ptr::read_volatile((bootloader_addr + 4) as *const u32);

            info!("Bootloader stack pointer: 0x{:08X}", stack_ptr);
            info!("Bootloader entry point: 0x{:08X}", reset_vector);

            // TODO: Validate stack pointer and reset vector values before using them
            // Set stack pointer
            cortex_m::register::msp::write(stack_ptr);

            // Jump to bootloader - this will not return
            let bootloader_entry: extern "C" fn() -> ! = core::mem::transmute(reset_vector);
            bootloader_entry();
        }
    }

    /// Get the unique hardware ID as a byte array
    /// Returns the device's unique identifier as raw bytes
    fn get_unique_id_bytes(&self) -> [u8; 12] {
        *embassy_stm32::uid::uid()
    }

    /// Get the unique hardware ID as a hexadecimal string
    /// Returns the device's unique identifier formatted as a hex string
    fn get_unique_id_hex(&self) -> heapless::String<24> {
        heapless::String::try_from(embassy_stm32::uid::uid_hex())
            .unwrap_or_else(|_| heapless::String::new())
    }
}

// Note: Default implementation removed because BlackPillDevice now requires
// peripherals to be passed in via new_with_peripherals() static method
