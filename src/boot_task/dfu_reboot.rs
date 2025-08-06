/// DFU reboot functionality module
/// This module provides hardware-agnostic DFU reboot functionality.
/// It handles the process of de-initializing the system and jumping to DFU mode
/// by delegating hardware-specific operations to the device trait implementation.

use crate::hw::traits::DeviceManagement;
use defmt::info;

/// Performs a complete system de-initialization and jumps to DFU mode.
/// 
/// This function must be called after clock setup and RTC setup, but before
/// any other peripherals are initialized. It will:
/// 1. Disable interrupts
/// 2. De-initialize RTC if it was initialized
/// 3. De-initialize clocks and prescalers
/// 4. Clear any pending interrupts
/// 5. Jump to the DFU bootloader
/// 
/// # Parameters
/// * `device` - A reference to a device that implements the DeviceManagement trait
/// 
/// # Safety
/// This function performs low-level system operations and will not return.
/// It should only be called when a DFU reboot is explicitly requested.
/// 
/// # Examples
/// ```
/// use sensor_swarm::boot_task::dfu_reboot::enter_dfu_mode;
/// use sensor_swarm::hw::blackpill_f401::device::BlackPillDevice;
/// 
/// let device = BlackPillDevice::new();
/// // This will not return
/// enter_dfu_mode(&device);
/// ```
pub fn enter_dfu_mode<T: for<'d> DeviceManagement<'d>>(device: &T) -> ! {
    info!("Initiating DFU reboot sequence...");
    
    // Step 1: Disable all interrupts to prevent interference
    device.disable_interrupts();
    
    // Step 2: De-initialize RTC
    device.deinitialize_rtc();
    
    // Step 3: De-initialize clocks and prescalers
    device.deinitialize_clocks();
    
    // Step 4: Clear any pending interrupts
    device.clear_pending_interrupts();
    
    // Step 5: Jump to DFU bootloader
    info!("Jumping to DFU bootloader...");
    device.jump_to_dfu_bootloader();
}
