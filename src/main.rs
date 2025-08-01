#![no_std]
#![no_main]

use defmt::info;

use panic_probe as _;

// Logging
#[cfg(not(test))]
use defmt_rtt as _;
#[cfg(test)]
use defmt_semihosting as _;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
// Import hardware abstraction and application logic
use embassy_usb::driver::EndpointError;
use sensor_swarm::app::SensorApp;
use sensor_swarm::hw::blackpill_f401::usb::UsbManager;
use sensor_swarm::hw::blackpill_f401::usb_defmt_logger::process_usb_log_queue;
use sensor_swarm::hw::traits::{DeviceManagement, Led};
use sensor_swarm::hw::BlackPillDevice;
use sensor_swarm::usb_log;

#[cfg(not(test))]
#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Starting sensor swarm application");

    // Initialize device manager
    let mut device_manager = BlackPillDevice::new();

    // Get the device-specific configuration for embassy initialization
    let embassy_config = device_manager.init().expect("Device initialization failed");
    let p = embassy_stm32::init(embassy_config);

    // Initialize LED first for early debugging (hardware-agnostic)
    let (mut led, remaining_peripherals) = device_manager
        .init_led(p)
        .expect("LED initialization failed");

    // Blink LED once to indicate LED initialization complete
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(1000).await;
    led.off();
    embassy_time::Timer::after_millis(1000).await;

    // Initialize USB components directly for proper Embassy task execution
    info!("Initializing USB components directly...");

    // Extract USB peripherals using unsafe pointer operations
    let (usb_otg_fs, pa12, pa11, _remaining_peripherals) = unsafe {
        let mut p = core::mem::ManuallyDrop::new(remaining_peripherals);
        let usb_otg_fs = core::ptr::read(&p.USB_OTG_FS);
        let pa12 = core::ptr::read(&p.PA12);
        let pa11 = core::ptr::read(&p.PA11);

        // Reconstruct peripherals without the extracted ones
        let remaining = core::ptr::read(&*p);
        (usb_otg_fs, pa12, pa11, remaining)
    };

    // Create USB manager for initialization
    let mut usb_manager = UsbManager::new();
    let (mut usb_device, mut cdc_class) = usb_manager
        .init_with_peripheral(usb_otg_fs, pa12, pa11)
        .await
        .expect("USB initialization failed");

    info!("Hardware peripherals initialized successfully");
    info!("USB device and CDC class ready for Embassy task execution");

    // Blink LED again to indicate all initialization complete
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    led.on();
    embassy_time::Timer::after_millis(200).await;
    led.off();
    embassy_time::Timer::after_millis(200).await;
    embassy_time::Timer::after_millis(1000).await;

    // Run USB device and CDC tasks concurrently using join() pattern
    let usb_fut = usb_device.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            cdc_class.wait_connection().await;
            info!("USB CDC connected!");
            let _ = echo(&mut cdc_class).await;
            info!("USB CDC disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
    unreachable!();
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: embassy_stm32::usb_otg::Instance + 'd>(
    class: &mut embassy_usb::class::cdc_acm::CdcAcmClass<'d, embassy_stm32::usb_otg::Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
