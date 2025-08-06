/// Hardware-in-the-Loop (HIL) testing utilities
/// This module provides hardware initialization and utilities for HIL tests
/// that require actual hardware peripherals and Embassy framework.

use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use crate::hw::blackpill_f401::device::BlackPillDevice;
use crate::hw::traits::{DeviceManagement, Led};

/// HIL test initialization result
pub struct HilTestContext {
    pub device: BlackPillDevice,
}

/// Initialize hardware for HIL testing
/// 
/// This function initializes real BlackPill hardware for HIL testing.
/// It takes Embassy STM32 peripherals and creates a properly configured BlackPill device.
/// 
/// # Returns
/// A `HilTestContext` containing the initialized real hardware device and spawner
/// 
/// # Note
/// This function requires Embassy STM32 peripherals to be available and will configure
/// the real BlackPill hardware including clocks, oscillators, and peripheral setup.
/// 
/// # Examples
/// ```no_run
/// #[cfg(feature = "hil")]
/// use sensor_swarm::testing::hil::init_hil_test;
/// 
/// #[cfg(feature = "hil")]
/// async fn test_led_blink() {
///     let ctx = init_hil_test().await;
///     // Use ctx.device for real hardware operations
/// }
/// ```
pub async fn init_hil_test() -> HilTestContext {
    // Initialize Embassy STM32 with BlackPill-specific configuration and get peripherals
    let peripherals = embassy_stm32::init(BlackPillDevice::get_embassy_config());
    
    // Create real BlackPill device with the peripherals
    let (_config, device) = BlackPillDevice::new_with_peripherals(peripherals)
        .expect("Failed to initialize BlackPill hardware for HIL testing");
    
    HilTestContext {
        device,
    }
}

/// Initialize hardware for HIL testing with custom configuration
/// 
/// This function allows for custom hardware configuration during HIL test
/// initialization using the provided Embassy STM32 configuration.
/// 
/// # Parameters
/// * `config` - Custom Embassy STM32 configuration to use for initialization
/// 
/// # Returns
/// A `HilTestContext` containing the initialized real hardware device and spawner
/// 
/// # Note
/// This function uses the provided configuration instead of the default BlackPill configuration.
pub async fn init_hil_test_with_config(config: Config) -> HilTestContext {
    // Initialize Embassy STM32 with the provided custom configuration and get peripherals
    let peripherals = embassy_stm32::init(config);
    
    // Create real BlackPill device with the peripherals
    let (_default_config, device) = BlackPillDevice::new_with_peripherals(peripherals)
        .expect("Failed to initialize BlackPill hardware for HIL testing");
    
    HilTestContext {
        device,
    }
}

/// Delay utility for HIL tests
/// 
/// Provides a convenient way to add delays in HIL tests using Embassy's
/// async timer functionality.
/// 
/// # Parameters
/// * `duration` - Duration to wait
/// 
/// # Examples
/// ```no_run
/// #[cfg(feature = "hil")]
/// use embassy_time::Duration;
/// #[cfg(feature = "hil")]
/// use sensor_swarm::testing::hil::hil_delay;
/// 
/// #[cfg(feature = "hil")]
/// async fn test_with_delay() {
///     hil_delay(Duration::from_millis(100)).await;
/// }
/// ```
pub async fn hil_delay(duration: Duration) {
    Timer::after(duration).await;
}

/// Basic LED test utility
/// 
/// This function provides a basic LED blinking test that can be used
/// to verify that HIL testing infrastructure is working correctly.
/// 
/// # Parameters
/// * `device` - Mutable reference to the device
/// * `blink_count` - Number of times to blink the LED
/// * `blink_duration` - Duration for each blink phase (on/off)
/// 
/// # Returns
/// `Ok(())` if the test completes successfully, `Err` if LED creation fails
/// 
/// # Examples
/// ```no_run
/// #[cfg(feature = "hil")]
/// use embassy_time::Duration;
/// #[cfg(feature = "hil")]
/// use sensor_swarm::testing::hil::{init_hil_test, test_led_blink};
/// 
/// #[cfg(feature = "hil")]
/// async fn run_led_test() {
///     let mut ctx = init_hil_test().await;
///     test_led_blink(&mut ctx.device, 3, Duration::from_millis(500)).await.unwrap();
/// }
/// ```
pub async fn test_led_blink(
    device: &mut BlackPillDevice, 
    blink_count: u32, 
    blink_duration: Duration
) -> Result<(), &'static str> {
    // Create LED
    let mut led = device.create_led()?;
    
    // Blink the LED
    for _ in 0..blink_count {
        led.on();
        hil_delay(blink_duration).await;
        led.off();
        hil_delay(blink_duration).await;
    }
    
    Ok(())
}

/// Test device information retrieval
/// 
/// This function tests that device information can be retrieved correctly
/// in a HIL environment. It's useful for verifying that the device is
/// properly initialized and accessible.
/// 
/// # Parameters
/// * `device` - Reference to the device
/// 
/// # Returns
/// `true` if device info retrieval is successful and contains expected values
pub fn test_device_info(device: &BlackPillDevice) -> bool {
    let info = device.get_device_info();
    
    // Basic sanity checks
    !info.model.is_empty() && 
    !info.board.is_empty() && 
    info.flash_size > 0 && 
    info.ram_size > 0 && 
    info.system_clock_hz > 0 &&
    !info.unique_id_hex.is_empty()
}

/// Initialize hardware for HIL testing (synchronous version)
/// 
/// This function initializes real BlackPill hardware for HIL testing synchronously.
/// This is the preferred version for defmt-test since it doesn't support async tests.
/// 
/// # Returns
/// A `HilTestContext` containing the initialized real hardware device and spawner
/// 
/// # Note
/// This function performs synchronous Embassy initialization which may not be suitable
/// for all use cases. For full async initialization, use `init_hil_test()` instead.
pub fn init_hil_test_sync() -> HilTestContext {
    // Initialize Embassy STM32 with BlackPill-specific configuration and get peripherals
    let peripherals = embassy_stm32::init(BlackPillDevice::get_embassy_config());
    
    // Create real BlackPill device with the peripherals
    let (_config, device) = BlackPillDevice::new_with_peripherals(peripherals)
        .expect("Failed to initialize BlackPill hardware for HIL testing");
    
    HilTestContext {
        device,
    }
}

/// Basic LED test utility (synchronous version)
/// 
/// This function provides a basic LED test that can be used
/// to verify that HIL testing infrastructure is working correctly.
/// 
/// # Parameters
/// * `device` - Mutable reference to the device
/// 
/// # Returns
/// `Ok(())` if the test completes successfully, `Err` if LED creation fails
pub fn test_led_basic(device: &mut BlackPillDevice) -> Result<(), &'static str> {
    // Create LED
    let mut led = device.create_led()?;
    
    // Test basic LED operations
    led.on();
    led.off();
    led.toggle();
    led.toggle();
    led.set_brightness(128);
    led.on();
    led.off();
    
    Ok(())
}
