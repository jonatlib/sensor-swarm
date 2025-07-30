/// Command Parser Module
///
/// This module handles parsing of USB commands and defines all supported command types.
/// It provides a clean interface for converting raw command strings into structured command enums.
use heapless::String;

/// Maximum command length in bytes
pub const MAX_COMMAND_LENGTH: usize = 256;

/// Represents different types of commands that can be sent over USB
#[derive(Debug, Clone, PartialEq)]
pub enum UsbCommand {
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
    /// Reboot the CPU
    RebootCpu,
    /// Reboot the CPU to DFU mode
    RebootCpuToDfu,
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

/// Command Parser
///
/// Handles parsing of raw command strings into structured UsbCommand enums.
/// Supports case-insensitive command matching.
pub struct CommandParser;

impl CommandParser {
    /// Create a new command parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a command from a byte buffer
    pub fn parse_command(&self, command_buffer: &[u8]) -> UsbCommand {
        // Convert buffer to string
        let command_str = match core::str::from_utf8(command_buffer) {
            Ok(s) => s.trim(),
            Err(_) => {
                let mut error_msg = String::new();
                let _ = error_msg.push_str("INVALID_UTF8");
                return UsbCommand::Unknown(error_msg);
            }
        };

        // Parse the command (case-insensitive comparison)
        // Helper function to compare strings case-insensitively
        let matches_command = |cmd: &str| -> bool {
            if command_str.len() != cmd.len() {
                return false;
            }
            command_str
                .chars()
                .zip(cmd.chars())
                .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
        };

        if matches_command("READ_SENSORS") || matches_command("SENSORS") {
            UsbCommand::ReadSensors
        } else if matches_command("READ_TEMPERATURE") || matches_command("TEMP") {
            UsbCommand::ReadSensorType(SensorType::Temperature)
        } else if matches_command("READ_HUMIDITY") || matches_command("HUMIDITY") {
            UsbCommand::ReadSensorType(SensorType::Humidity)
        } else if matches_command("READ_LIGHT") || matches_command("LIGHT") {
            UsbCommand::ReadSensorType(SensorType::Light)
        } else if matches_command("READ_PRESSURE") || matches_command("PRESSURE") {
            UsbCommand::ReadSensorType(SensorType::Pressure)
        } else if matches_command("DEBUG") || matches_command("DEBUG_INFO") {
            UsbCommand::GetDebugInfo
        } else if matches_command("STATUS") {
            UsbCommand::GetStatus
        } else if matches_command("PING") {
            UsbCommand::Ping
        } else if matches_command("HELP") || matches_command("?") {
            UsbCommand::Help
        } else if matches_command("REBOOT") || matches_command("REBOOT_CPU") {
            UsbCommand::RebootCpu
        } else if matches_command("REBOOT_DFU")
            || matches_command("REBOOT_CPU_DFU")
            || matches_command("DFU")
        {
            UsbCommand::RebootCpuToDfu
        } else {
            let mut unknown_cmd = String::new();
            let _ = unknown_cmd.push_str(command_str);
            UsbCommand::Unknown(unknown_cmd)
        }
    }

    /// Get help text for all supported commands
    pub fn get_help_text(&self) -> String<256> {
        let mut help_text = String::new();
        let _ = help_text.push_str("Available commands:\n");
        let _ = help_text.push_str("SENSORS - Read all sensor data\n");
        let _ = help_text.push_str("TEMP - Read temperature\n");
        let _ = help_text.push_str("HUMIDITY - Read humidity\n");
        let _ = help_text.push_str("LIGHT - Read light level\n");
        let _ = help_text.push_str("PRESSURE - Read pressure\n");
        let _ = help_text.push_str("DEBUG - Get debug info\n");
        let _ = help_text.push_str("STATUS - Get device status\n");
        let _ = help_text.push_str("PING - Test connectivity\n");
        let _ = help_text.push_str("REBOOT - Reboot CPU\n");
        let _ = help_text.push_str("DFU - Reboot to DFU mode\n");
        let _ = help_text.push_str("HELP - Show this help");
        help_text
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}
