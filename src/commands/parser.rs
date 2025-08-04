/// Command parsing module
/// This module handles parsing command strings into structured Command enums

use heapless::String;

/// Represents different types of commands that can be sent over terminal
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Read all available sensor data
    ReadSensors,
    /// Read specific sensor data by type
    ReadSensorType(SensorType),
    /// Get device debug information
    GetDebugInfo,
    /// Get device status
    GetStatus,
    /// Ping command for connectivity testing
    Ping,
    /// Get list of available commands
    Help,
    /// Show firmware version
    Version,
    /// Reboot the CPU
    Reboot,
    /// Reboot the CPU to DFU mode
    RebootToDfu,
    /// Unknown/invalid command
    Unknown(String<64>),
}

/// Types of sensors that can be queried individually
#[derive(Debug, Clone, PartialEq)]
pub enum SensorType {
    Temperature,
    Humidity,
    Light,
    Pressure,
}

/// Command parser that converts string commands into Command enums
pub struct CommandParser;

impl CommandParser {
    /// Create a new command parser
    pub fn new() -> Self {
        Self
    }

    /// Parse command string into Command enum
    pub fn parse(&self, command_str: &str) -> Command {
        // Helper function to compare strings case-insensitively
        let matches_command = |cmd: &str| -> bool {
            if command_str.len() != cmd.len() {
                return false;
            }
            command_str
                .chars()
                .zip(cmd.chars())
                .all(|(a, b)| a.eq_ignore_ascii_case(&b))
        };

        if matches_command("sensors") || matches_command("read_sensors") {
            Command::ReadSensors
        } else if matches_command("temp") || matches_command("temperature") {
            Command::ReadSensorType(SensorType::Temperature)
        } else if matches_command("humidity") {
            Command::ReadSensorType(SensorType::Humidity)
        } else if matches_command("light") {
            Command::ReadSensorType(SensorType::Light)
        } else if matches_command("pressure") {
            Command::ReadSensorType(SensorType::Pressure)
        } else if matches_command("debug") || matches_command("debug_info") {
            Command::GetDebugInfo
        } else if matches_command("status") {
            Command::GetStatus
        } else if matches_command("ping") {
            Command::Ping
        } else if matches_command("help") || matches_command("?") {
            Command::Help
        } else if matches_command("version") {
            Command::Version
        } else if matches_command("reboot") {
            Command::Reboot
        } else if matches_command("dfu") || matches_command("reboot_dfu") {
            Command::RebootToDfu
        } else {
            let mut unknown_cmd = String::new();
            let _ = unknown_cmd.push_str(command_str);
            Command::Unknown(unknown_cmd)
        }
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}