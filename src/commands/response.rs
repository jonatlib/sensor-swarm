/// Command response module
/// This module defines response types and their formatting for command execution
use super::parser::SensorType;
use crate::hw::traits::DeviceInfo;
use core::fmt;
use heapless::String;

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
    Error { message: String<128> },
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
            SensorValue::Temperature(temp) => write!(f, "{temp}°C"),
            SensorValue::Humidity(hum) => write!(f, "{hum}%"),
            SensorValue::Light(light) => write!(f, "{light} lux"),
            SensorValue::Pressure(pressure) => write!(f, "{pressure} hPa"),
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
                writeln!(f, "Available commands:")?;
                writeln!(f, "  help - Show this help message")?;
                writeln!(f, "  sensors - Read all sensor data")?;
                writeln!(f, "  temp - Read temperature")?;
                writeln!(f, "  humidity - Read humidity")?;
                writeln!(f, "  light - Read light level")?;
                writeln!(f, "  pressure - Read pressure")?;
                writeln!(f, "  debug - Get debug info")?;
                writeln!(f, "  device - Show device information")?;
                writeln!(f, "  status - Show device status")?;
                writeln!(f, "  ping - Test connectivity")?;
                writeln!(f, "  version - Show firmware version")?;
                writeln!(f, "  reboot - Reboot the device")?;
                write!(f, "  dfu - Reboot to DFU mode")
            }
            Response::Status {
                usb_connected,
                terminal_active,
                system_running,
            } => {
                writeln!(f, "Device Status:")?;
                writeln!(
                    f,
                    "  USB: {}",
                    if *usb_connected {
                        "Connected"
                    } else {
                        "Disconnected"
                    }
                )?;
                writeln!(
                    f,
                    "  Terminal: {}",
                    if *terminal_active {
                        "Active"
                    } else {
                        "Inactive"
                    }
                )?;
                write!(
                    f,
                    "  System: {}",
                    if *system_running {
                        "Running"
                    } else {
                        "Stopped"
                    }
                )
            }
            Response::Version {
                version,
                description,
            } => {
                write!(f, "{version}\n{description}")
            }
            Response::Ping => {
                write!(f, "PONG - Terminal connection active")
            }
            Response::AllSensors {
                temperature,
                humidity,
                light,
                pressure,
            } => {
                writeln!(f, "Reading all sensors...")?;
                writeln!(f, "Temperature: {temperature}°C")?;
                writeln!(f, "Humidity: {humidity}%")?;
                writeln!(f, "Light: {light} lux")?;
                write!(f, "Pressure: {pressure} hPa")
            }
            Response::SensorReading { sensor_type, value } => match sensor_type {
                SensorType::Temperature => write!(f, "Temperature: {value}"),
                SensorType::Humidity => write!(f, "Humidity: {value}"),
                SensorType::Light => write!(f, "Light: {value}"),
                SensorType::Pressure => write!(f, "Pressure: {value}"),
            },
            Response::Debug {
                uptime_ms,
                free_memory,
                usb_connected,
                sensor_count,
            } => {
                writeln!(f, "Debug Information:")?;
                writeln!(f, "  Uptime: {uptime_ms} ms")?;
                writeln!(f, "  Free Memory: {free_memory} bytes")?;
                writeln!(f, "  USB Connected: {usb_connected}")?;
                write!(f, "  Sensors: {sensor_count} available")
            }
            Response::DeviceInfo {
                model,
                board,
                flash_size,
                ram_size,
                system_clock_hz,
                usb_clock_hz,
                unique_id_hex,
            } => {
                writeln!(f, "Device Information:")?;
                writeln!(f, "  Model: {model}")?;
                writeln!(f, "  Board: {board}")?;
                writeln!(f, "  Flash Size: {} KB", flash_size / 1024)?;
                writeln!(f, "  RAM Size: {} KB", ram_size / 1024)?;
                writeln!(f, "  System Clock: {} MHz", system_clock_hz / 1_000_000)?;
                writeln!(f, "  USB Clock: {} MHz", usb_clock_hz / 1_000_000)?;
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
