/// Command execution module
/// This module handles executing parsed commands and generating responses

use super::parser::{Command, SensorType};
use heapless::String;
use core::fmt;
use crate::hw::traits::{DeviceManagement, BackupRegisters, DeviceInfo};
use crate::hw::{BootTask, BackupRegister};

/// Response enum representing different types of command responses
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Help message with available commands
    Help,
    /// Device status information
    Status {
        usb_connected: bool,
        terminal_active: bool,
        system_running: bool,
    },
    /// Firmware version information
    Version {
        version: &'static str,
        description: &'static str,
    },
    /// Ping response
    Ping,
    /// All sensor readings
    AllSensors {
        temperature: f32,
        humidity: u8,
        light: u16,
        pressure: u16,
    },
    /// Individual sensor reading
    SensorReading {
        sensor_type: SensorType,
        value: SensorValue,
    },
    /// Debug information
    Debug {
        uptime_ms: u32,
        free_memory: u32,
        usb_connected: bool,
        sensor_count: u8,
    },
    /// Device information
    DeviceInfo {
        model: &'static str,
        board: &'static str,
        flash_size: u32,
        ram_size: u32,
        system_clock_hz: u32,
        usb_clock_hz: u32,
        unique_id_hex: heapless::String<24>,
    },
    /// Reboot confirmation
    Reboot,
    /// DFU reboot confirmation
    RebootToDfu,
    /// Error for unknown commands
    Error {
        message: String<128>,
    },
}

/// Sensor value types
#[derive(Debug, Clone, PartialEq)]
pub enum SensorValue {
    Temperature(f32),
    Humidity(u8),
    Light(u16),
    Pressure(u16),
}

impl fmt::Display for SensorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorValue::Temperature(temp) => write!(f, "{}°C", temp),
            SensorValue::Humidity(hum) => write!(f, "{}%", hum),
            SensorValue::Light(light) => write!(f, "{} lux", light),
            SensorValue::Pressure(pressure) => write!(f, "{} hPa", pressure),
        }
    }
}

/// Implement From trait to convert DeviceInfo to Response::DeviceInfo
impl From<DeviceInfo> for Response {
    fn from(device_info: DeviceInfo) -> Self {
        Response::DeviceInfo {
            model: device_info.model,
            board: device_info.board,
            flash_size: device_info.flash_size,
            ram_size: device_info.ram_size,
            system_clock_hz: device_info.system_clock_hz,
            usb_clock_hz: device_info.usb_clock_hz,
            unique_id_hex: device_info.unique_id_hex,
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Help => {
                write!(f, "Available commands:\n")?;
                write!(f, "  help - Show this help message\n")?;
                write!(f, "  sensors - Read all sensor data\n")?;
                write!(f, "  temp - Read temperature\n")?;
                write!(f, "  humidity - Read humidity\n")?;
                write!(f, "  light - Read light level\n")?;
                write!(f, "  pressure - Read pressure\n")?;
                write!(f, "  debug - Get debug info\n")?;
                write!(f, "  device - Show device information\n")?;
                write!(f, "  status - Show device status\n")?;
                write!(f, "  ping - Test connectivity\n")?;
                write!(f, "  version - Show firmware version\n")?;
                write!(f, "  reboot - Reboot the device\n")?;
                write!(f, "  dfu - Reboot to DFU mode")
            }
            Response::Status { usb_connected, terminal_active, system_running } => {
                write!(f, "Device Status:\n")?;
                write!(f, "  USB: {}\n", if *usb_connected { "Connected" } else { "Disconnected" })?;
                write!(f, "  Terminal: {}\n", if *terminal_active { "Active" } else { "Inactive" })?;
                write!(f, "  System: {}", if *system_running { "Running" } else { "Stopped" })
            }
            Response::Version { version, description } => {
                write!(f, "{}\n{}", version, description)
            }
            Response::Ping => {
                write!(f, "PONG - Terminal connection active")
            }
            Response::AllSensors { temperature, humidity, light, pressure } => {
                write!(f, "Reading all sensors...\n")?;
                write!(f, "Temperature: {}°C\n", temperature)?;
                write!(f, "Humidity: {}%\n", humidity)?;
                write!(f, "Light: {} lux\n", light)?;
                write!(f, "Pressure: {} hPa", pressure)
            }
            Response::SensorReading { sensor_type, value } => {
                match sensor_type {
                    SensorType::Temperature => write!(f, "Temperature: {}", value),
                    SensorType::Humidity => write!(f, "Humidity: {}", value),
                    SensorType::Light => write!(f, "Light: {}", value),
                    SensorType::Pressure => write!(f, "Pressure: {}", value),
                }
            }
            Response::Debug { uptime_ms, free_memory, usb_connected, sensor_count } => {
                write!(f, "Debug Information:\n")?;
                write!(f, "  Uptime: {} ms\n", uptime_ms)?;
                write!(f, "  Free Memory: {} bytes\n", free_memory)?;
                write!(f, "  USB Connected: {}\n", usb_connected)?;
                write!(f, "  Sensors: {} available", sensor_count)
            }
            Response::DeviceInfo { model, board, flash_size, ram_size, system_clock_hz, usb_clock_hz, unique_id_hex } => {
                write!(f, "Device Information:\n")?;
                write!(f, "  Model: {}\n", model)?;
                write!(f, "  Board: {}\n", board)?;
                write!(f, "  Flash Size: {} KB\n", flash_size / 1024)?;
                write!(f, "  RAM Size: {} KB\n", ram_size / 1024)?;
                write!(f, "  System Clock: {} MHz\n", system_clock_hz / 1_000_000)?;
                write!(f, "  USB Clock: {} MHz\n", usb_clock_hz / 1_000_000)?;
                write!(f, "  Unique ID: {}", unique_id_hex.as_str())
            }
            Response::Reboot => {
                write!(f, "Rebooting device...")
            }
            Response::RebootToDfu => {
                write!(f, "Rebooting to DFU mode...")
            }
            Response::Error { message } => {
                write!(f, "{}", message.as_str())
            }
        }
    }
}

/// Command executor that runs commands and generates responses
/// 
/// This executor processes parsed commands and generates appropriate responses.
/// It uses a device manager to access hardware functionality where implemented,
/// but currently contains placeholder implementations for many features.
/// 
/// # Type Parameters
/// * `D` - Device management implementation that provides hardware abstraction
/// 
/// # Current Implementation Status
/// - Device information: ✓ Fully implemented
/// - Help and basic commands: ✓ Fully implemented  
/// - Sensor readings: ❌ Hardcoded values (needs sensor integration)
/// - Status monitoring: ❌ Hardcoded values (needs status tracking)
/// - Debug information: ❌ Hardcoded values (needs system monitoring)
/// - Reboot commands: ⚠️ Response-only (needs delayed execution mechanism)
pub struct CommandExecutor<D: for<'d> DeviceManagement<'d>> {
    device_manager: D,
}

impl<D: for<'d> DeviceManagement<'d>> CommandExecutor<D> {
    /// Create a new command executor
    pub fn new(device_manager: D) -> Self {
        Self { device_manager }
    }

    /// Execute a parsed command and return response
    /// 
    /// This method processes commands and generates appropriate responses.
    /// Some commands use hardcoded values as placeholders until proper
    /// hardware integration is implemented.
    pub async fn execute(&mut self, command: Command) -> Response {
        match command {
            Command::Help => Response::Help,
            
            Command::GetStatus => {
                // TODO: Implement actual status checking
                // Currently returns hardcoded values - need to implement:
                // - USB connection status detection
                // - Terminal activity monitoring  
                // - System health monitoring
                Response::Status {
                    usb_connected: true,  // FIXME: Hardcoded - should check actual USB status
                    terminal_active: true,  // FIXME: Hardcoded - should check terminal state
                    system_running: true,  // FIXME: Hardcoded - should check system health
                }
            },
            
            Command::Version => Response::Version {
                version: "Sensor Swarm Firmware v1.0.0",
                description: "Built with modular command architecture",
            },
            
            Command::Ping => Response::Ping,
            
            Command::ReadSensors => {
                // TODO: Implement actual sensor reading
                // Need to integrate with EnvironmentalSensor trait from sensors::traits
                // Currently returns hardcoded test values
                Response::AllSensors {
                    temperature: 25.0,  // FIXME: Hardcoded test value
                    humidity: 60,       // FIXME: Hardcoded test value  
                    light: 1000,        // FIXME: Hardcoded test value
                    pressure: 1013,     // FIXME: Hardcoded test value
                }
            },
            
            Command::ReadSensorType(sensor_type) => {
                // TODO: Implement actual individual sensor reading
                // Need to integrate with EnvironmentalSensor trait from sensors::traits
                // Currently returns hardcoded test values
                let value = match sensor_type {
                    SensorType::Temperature => SensorValue::Temperature(25.0),  // FIXME: Hardcoded
                    SensorType::Humidity => SensorValue::Humidity(60),          // FIXME: Hardcoded
                    SensorType::Light => SensorValue::Light(1000),              // FIXME: Hardcoded
                    SensorType::Pressure => SensorValue::Pressure(1013),        // FIXME: Hardcoded
                };
                Response::SensorReading { sensor_type, value }
            }
            
            Command::GetDebugInfo => {
                // TODO: Implement actual debug information gathering
                // Need to implement:
                // - Actual uptime tracking
                // - Memory usage monitoring
                // - USB connection status
                // - Sensor availability detection
                Response::Debug {
                    uptime_ms: 12345,      // FIXME: Hardcoded - need uptime tracking
                    free_memory: 8192,     // FIXME: Hardcoded - need memory monitoring
                    usb_connected: true,   // FIXME: Hardcoded - should check USB status
                    sensor_count: 4,       // FIXME: Hardcoded - should count available sensors
                }
            },
            
            Command::GetDeviceInfo => {
                // This command is properly implemented using device_manager
                let device_info = self.device_manager.get_device_info();
                device_info.into()
            },
            
            Command::Reboot => {
                // Note: This will reboot the device and never return
                // We can't return a Response because the method never returns
                self.device_manager.reboot();
            }
            
            Command::RebootToDfu => {
                // Register DFU boot task in backup domain and reboot
                // This is safer than directly jumping to DFU bootloader
                if let Some(backup_registers) = self.device_manager.get_backup_registers() {
                    // Write DFU boot task to backup register
                    backup_registers.write_register(BackupRegister::BootTask as usize, BootTask::DFUReboot as u32);
                    
                    // Now reboot - the boot task will be handled on next startup
                    self.device_manager.reboot();
                } else {
                    // Fallback to direct DFU jump if backup registers not available
                    self.device_manager.jump_to_dfu_bootloader();
                }
            }
            Command::Unknown(cmd) => {
                let mut message = String::new();
                let _ = core::fmt::write(&mut message, format_args!("Error: Unknown command '{}'. Type 'help' for available commands.", cmd.as_str()))
                    .map_err(|_| todo!("Handle string formatting error"));
                Response::Error { message }
            }
        }
    }

    /// Convert response to string for backward compatibility
    /// 
    /// This method converts a Response enum to a formatted string representation.
    /// It's provided for backward compatibility with systems that expect string responses.
    /// 
    /// # Parameters
    /// * `response` - The response to convert to string format
    /// 
    /// # Returns
    /// A heapless String containing the formatted response text
    /// 
    /// # Note
    /// This method uses the Display implementation of Response for formatting.
    /// Formatting errors are silently ignored as they're unlikely in this context.
    pub fn response_to_string(&self, response: &Response) -> String<512> {
        let mut result = String::new();
        let _ = core::fmt::write(&mut result, format_args!("{}", response))
            .map_err(|_| todo!("Handle response formatting error"));
        result
    }
}
