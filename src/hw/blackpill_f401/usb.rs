/// USB Manager implementation for STM32F401 Black Pill
/// Provides USB-based debugging and device management functionality

use crate::hw::traits::{DebugInterface, DeviceManagement};
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, UsbDevice};
use defmt::*;

// Bind USB interrupt
bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

/// USB Manager for STM32F401 Black Pill
/// Handles USB Serial communication for debugging and DFU bootloader access
pub struct UsbManager {
    usb_device: Option<UsbDevice<'static, Driver<'static, peripherals::USB_OTG_FS>>>,
    cdc_class: Option<CdcAcmClass<'static, Driver<'static, peripherals::USB_OTG_FS>>>,
}

impl UsbManager {
    /// Create a new USB Manager instance
    pub fn new() -> Self {
        Self {
            usb_device: None,
            cdc_class: None,
        }
    }

    /// Initialize USB peripheral with the given USB OTG FS peripheral
    pub async fn init_with_peripheral(
        &mut self,
        usb: peripherals::USB_OTG_FS,
        dp: peripherals::PA12,
        dm: peripherals::PA11,
    ) -> Result<(), &'static str> {
        info!("Initializing USB peripheral...");

        // Create the driver
        let mut config = Config::default();
        config.rcc.hse = Some(embassy_stm32::rcc::Hse {
            freq: embassy_stm32::time::Hertz(25_000_000),
            mode: embassy_stm32::rcc::HseMode::Oscillator,
        });
        config.rcc.pll_src = embassy_stm32::rcc::PllSource::HSE;
        config.rcc.pll = Some(embassy_stm32::rcc::Pll {
            prediv: embassy_stm32::rcc::PllPreDiv::DIV25,
            mul: embassy_stm32::rcc::PllMul::MUL336,
            divp: Some(embassy_stm32::rcc::PllPDiv::DIV4), // 336 MHz / 4 = 84 MHz
            divq: Some(embassy_stm32::rcc::PllQDiv::DIV7), // 336 MHz / 7 = 48 MHz (USB)
            divr: None,
        });
        config.rcc.ahb_pre = embassy_stm32::rcc::AHBPrescaler::DIV1;
        config.rcc.apb1_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
        config.rcc.apb2_pre = embassy_stm32::rcc::APBPrescaler::DIV1;
        config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_P;

        let driver = Driver::new_fs(usb, Irqs, dp, dm);

        // Create embassy-usb Config
        let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
        usb_config.manufacturer = Some("Sensor Swarm");
        usb_config.product = Some("Debug Interface");
        usb_config.serial_number = Some("12345678");
        usb_config.max_power = 100;
        usb_config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        usb_config.device_class = 0xEF;
        usb_config.device_sub_class = 0x02;
        usb_config.device_protocol = 0x01;
        usb_config.composite_with_iads = true;

        // Create embassy-usb DeviceBuilder
        let mut device_descriptor = [0; 256];
        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut control_buf = [0; 64];

        let mut state = State::new();
        let mut builder = Builder::new(
            driver,
            usb_config,
            &mut device_descriptor,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [], // no msos descriptors
            &mut control_buf,
        );

        // Create classes on the builder.
        let cdc_class = CdcAcmClass::new(&mut builder, &mut state, 64);

        // Build the builder.
        let usb_device = builder.build();

        self.usb_device = Some(usb_device);
        self.cdc_class = Some(cdc_class);

        info!("USB peripheral initialized successfully");
        Ok(())
    }
}

impl DebugInterface for UsbManager {
    /// Initialize the debug interface over USB Serial
    fn init(&mut self) -> impl core::future::Future<Output = Result<(), &'static str>> + Send {
        async move {
            info!("Setting up USB Serial debug interface...");
            
            // Note: The actual USB device initialization should be done via init_with_peripheral
            // This method confirms the debug interface is ready
            if self.usb_device.is_some() && self.cdc_class.is_some() {
                info!("USB Serial debug interface ready");
                Ok(())
            } else {
                error!("USB peripheral not initialized. Call init_with_peripheral first.");
                Err("USB peripheral not initialized")
            }
        }
    }
}

impl DeviceManagement for UsbManager {
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

impl Default for UsbManager {
    fn default() -> Self {
        Self::new()
    }
}