use crate::hw::pipico::backup_registers::PiPicoBackupRegisters;
use crate::hw::pipico::led::PiPicoLed;
use crate::hw::pipico::usb::UsbManager;
/// Device initialization and management for Raspberry Pi Pico (RP2040)
/// Provides hardware-specific device setup and configuration
use crate::hw::traits::{DeviceInfo, DeviceManagement};
use defmt::{info, warn};

/// Device manager for Raspberry Pi Pico (RP2040)
/// Handles device initialization, clock configuration, and system management
/// Stores peripherals individually to enable safe peripheral creation with lifetimes
pub struct PiPicoDevice {
    // Store individual peripherals as Options to allow safe extraction
    pin25: Option<embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_25>>, // Built-in LED on Pico
    usb: Option<embassy_rp::Peri<'static, embassy_rp::peripherals::USB>>,
    pin0: Option<embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_0>>,   // GPIO0 for general use
    pin1: Option<embassy_rp::Peri<'static, embassy_rp::peripherals::PIN_1>>,   // GPIO1 for general use
    rtc: Option<embassy_rp::Peri<'static, embassy_rp::peripherals::RTC>>,
    backup_registers: Option<PiPicoBackupRegisters>,
}

impl PiPicoDevice {
    /// Create a new device manager instance with peripherals stored internally
    /// This replaces the old unsafe peripheral-passing pattern
    fn new_internal(peripherals: embassy_rp::Peripherals) -> Self {
        Self {
            pin25: Some(peripherals.PIN_25.into()),
            usb: Some(peripherals.USB.into()),
            pin0: Some(peripherals.PIN_0.into()),
            pin1: Some(peripherals.PIN_1.into()),
            rtc: Some(peripherals.RTC.into()),
            backup_registers: None,
        }
    }

    /// Get the Embassy configuration for Raspberry Pi Pico (RP2040)
    /// This is now a static method that doesn't require a device instance
    pub fn get_embassy_config() -> embassy_rp::config::Config {
        let mut config = embassy_rp::config::Config::default();
        
        // RP2040 runs at 125MHz by default with internal oscillator
        // The Pico has a 12MHz crystal, but we'll use the default configuration
        // which should work well for most applications
        
        config
    }
}

impl<'d> DeviceManagement<'d> for PiPicoDevice {
    /// LED type - using PiPicoLed for PIN_25 (built-in LED)
    type Led = PiPicoLed;
    /// USB Wrapper type - dummy UsbCdcWrapper for terminal usage
    type UsbWrapper = crate::usb::UsbCdcWrapper;
    /// BackupRegisters type - using PiPicoBackupRegisters for RTC backup registers
    type BackupRegisters = PiPicoBackupRegisters;
    /// Peripheral type for RP2040
    type Peripherals = embassy_rp::Peripherals;
    /// Config type for RP2040
    type Config = embassy_rp::config::Config;

    /// Create a new device manager instance with peripherals stored internally
    /// This static method returns the Embassy configuration and creates the device manager
    /// with the peripherals stored internally, eliminating unsafe pointer operations
    fn new_with_peripherals(
        peripherals: Self::Peripherals,
    ) -> Result<(Self::Config, Self), &'static str> {
        let config = Self::get_embassy_config();
        let device = Self::new_internal(peripherals);
        Ok((config, device))
    }

    /// Get device information
    fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            model: "RP2040",
            board: "Raspberry Pi Pico",
            flash_size: 2 * 1024 * 1024,    // 2MB external flash
            ram_size: 264 * 1024,           // 264KB SRAM
            system_clock_hz: 125_000_000,   // 125MHz default system clock
            usb_clock_hz: 48_000_000,       // 48MHz USB clock
            unique_id_hex: self.get_unique_id_hex(),
        }
    }

    /// Perform a soft reset of the device
    fn soft_reset(&self) -> ! {
        info!("Performing soft reset...");
        cortex_m::peripheral::SCB::sys_reset();
        // This should never be reached, but the compiler needs explicit never-return
        unreachable!()
    }

    /// Create LED peripheral from stored peripherals for early debugging
    /// This method safely extracts PIN_25 from the internally stored peripherals
    fn create_led(&'d mut self) -> Result<Self::Led, &'static str> {
        let pin25 = self
            .pin25
            .take()
            .ok_or("PIN_25 peripheral already used or not available")?;

        info!("Creating LED on PIN_25 (built-in LED)");
        
        PiPicoLed::new(pin25)
    }

    /// Create USB peripheral from stored peripherals
    /// This method safely extracts USB from the internally stored peripherals
    fn create_usb(
        &'d mut self,
    ) -> impl core::future::Future<Output = Result<Self::UsbWrapper, &'static str>> + Send {
        async move {
            let usb = self
                .usb
                .take()
                .ok_or("USB peripheral already used or not available")?;

            info!("Creating dummy USB CDC wrapper for RP2040");

            // Initialize UsbManager and create a dummy CDC wrapper
            let manager = UsbManager::new(usb)?;
            let wrapper = manager.create_cdc_wrapper().await?;
            Ok(wrapper)
        }
    }

    /// Create RTC peripheral and backup registers from stored peripherals
    /// This method safely extracts RTC from the internally stored peripherals
    fn create_rtc(&'d mut self) -> Result<Self::BackupRegisters, &'static str> {
        let rtc = self
            .rtc
            .take()
            .ok_or("RTC peripheral already used or not available")?;

        info!("Creating RTC and backup registers");
        
        // Create backup registers instance and return it
        // The caller is responsible for storing it if needed
        let backup_registers = PiPicoBackupRegisters::new(*rtc)?;
        
        info!("RTC and backup registers initialized successfully");
        Ok(backup_registers)
    }

    /// Get access to backup registers for boot task management
    fn get_backup_registers(&mut self) -> Option<&mut Self::BackupRegisters> {
        self.backup_registers.as_mut()
    }

    /// Reboot the device normally
    fn reboot(&self) -> ! {
        info!("Rebooting device...");
        // TODO: Implement proper RP2040 reboot mechanism
        cortex_m::peripheral::SCB::sys_reset();
        // This should never be reached, but the compiler needs explicit never-return
        unreachable!()
    }

    /// Disable all interrupts to prevent interference during DFU transition
    fn disable_interrupts(&self) {
        info!("Disabling interrupts");
        cortex_m::interrupt::disable();
        
        // TODO: Disable RP2040-specific interrupts if needed
        // FIXME: Add RP2040-specific interrupt disabling
    }

    /// De-initialize the RTC peripheral
    fn deinitialize_rtc(&self) {
        info!("De-initializing RTC");
        // TODO: Implement RTC de-initialization for RP2040
        // FIXME: Add proper RTC de-initialization
    }

    /// De-initialize system clocks and prescalers
    fn deinitialize_clocks(&self) {
        info!("De-initializing clocks");
        // TODO: Implement clock de-initialization for RP2040
        // FIXME: Add proper clock de-initialization
    }

    /// Clear any pending interrupts
    fn clear_pending_interrupts(&self) {
        info!("Clearing pending interrupts");
        // Clear all pending interrupts in NVIC
        unsafe {
            let nvic = &*cortex_m::peripheral::NVIC::PTR;
            // Clear all pending interrupts (RP2040 has 32 interrupts)
            nvic.icpr[0].write(0xFFFFFFFF);
        }
    }

    /// Jump to the bootloader without resetting the device
    /// For RP2040, this involves entering BOOTSEL mode
    fn jump_to_dfu_bootloader(&self) -> ! {
        info!("Jumping to RP2040 bootloader (BOOTSEL mode)");
        
        // TODO: Implement proper RP2040 bootloader entry
        // The RP2040 bootloader can be entered by:
        // 1. Holding BOOTSEL button during reset
        // 2. Writing specific values to watchdog scratch registers and resetting
        // FIXME: Implement proper BOOTSEL mode entry
        
        // For now, just reset - user will need to manually enter BOOTSEL mode
        warn!("RP2040 bootloader entry not fully implemented - performing reset");
        cortex_m::peripheral::SCB::sys_reset();
        // This should never be reached, but the compiler needs explicit never-return
        unreachable!()
    }

    /// Get the unique hardware ID as a byte array
    /// RP2040 has a unique 64-bit ID, we'll use the first 12 bytes (96 bits)
    fn get_unique_id_bytes(&self) -> [u8; 12] {
        // TODO: Implement proper RP2040 unique ID reading
        // RP2040 unique ID is stored in OTP memory
        // FIXME: Read actual unique ID from RP2040 OTP
        
        // For now, return a placeholder
        [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B]
    }

    /// Get the unique hardware ID as a hexadecimal string
    fn get_unique_id_hex(&self) -> heapless::String<24> {
        let bytes = self.get_unique_id_bytes();
        let mut hex_string = heapless::String::<24>::new();
        
        for byte in bytes.iter() {
            let _ = core::fmt::write(&mut hex_string, format_args!("{:02X}", byte));
        }
        
        hex_string
    }
}

/// Initialize embassy with current device configuration
/// Returns the embassy peripherals for Raspberry Pi Pico (RP2040)
pub fn init_embassy() -> embassy_rp::Peripherals {
    embassy_rp::init(PiPicoDevice::get_embassy_config())
}