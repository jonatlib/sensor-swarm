/// Boot task execution module
/// This module handles the execution of boot tasks that are stored in backup registers
/// and need to be performed after a device reset.

pub mod dfu_reboot;

use defmt::info;
use crate::hw::BootTask;
use crate::hw::traits::DeviceManagement;

/// Execute a boot task based on the provided BootTask enum value.
/// This function handles the different types of boot tasks that can be requested
/// after a device reset, such as firmware updates or self-tests.
/// 
/// # Arguments
/// * `boot_task` - The BootTask enum value indicating which task to execute
/// * `device` - The device manager that implements DeviceManagement trait
/// 
/// # Examples
/// ```
/// use sensor_swarm::boot_task::execute_boot_task;
/// use sensor_swarm::hw::BootTask;
/// use sensor_swarm::hw::blackpill_f401::device::BlackPillDevice;
/// 
/// let device = BlackPillDevice::new();
/// // Execute a firmware update task
/// execute_boot_task(BootTask::UpdateFirmware, &device);
/// 
/// // Handle normal boot (no special task)
/// execute_boot_task(BootTask::None, &device);
/// ```
pub fn execute_boot_task<T: for<'d> DeviceManagement<'d>>(boot_task: BootTask, device: &T) {
    info!("Executing boot task: {:?}", boot_task);
    
    // Execute the boot task based on its type
    match boot_task {
        BootTask::None => {
            info!("Normal boot - no special tasks to execute");
        }
        BootTask::UpdateFirmware => {
            info!("Executing FIRMWARE UPDATE task...");
            // In a real implementation, this would trigger firmware update logic
            // For now, we just log the action
            info!("Firmware update task completed (stub implementation)");
        }
        BootTask::RunSelfTest => {
            info!("Executing SELF-TEST task...");
            // In a real implementation, this would run comprehensive self-tests
            // For now, we just log the action
            info!("Self-test task completed (stub implementation)");
        }
        BootTask::DFUReboot => {
            info!("Executing DFU REBOOT task...");
            // This will de-initialize the system and jump to DFU mode
            // This function will not return
            dfu_reboot::enter_dfu_mode(device);
        }
    }
    
    info!("Boot task execution completed");
}
