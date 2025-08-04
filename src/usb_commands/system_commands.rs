/// System Commands Handler Module
///
/// This module handles all system-related USB commands including debug info,
/// device status, ping, and help commands. It provides system-level information
/// and utility functions.
use crate::terminal::Terminal;
use crate::usb::UsbCdc;
use crate::usb_commands::parser::{CommandParser, UsbCommand};
use crate::usb_commands::responses::{DebugInfo, DeviceStatus, UsbResponse};
use heapless::String;

/// System Commands Handler
///
/// Handles processing of system-related commands and generates appropriate responses.
pub struct SystemCommandHandler {
    parser: CommandParser,
    uptime_start: u64, // For tracking uptime
}

impl SystemCommandHandler {
    /// Create a new system command handler
    pub fn new() -> Self {
        Self {
            parser: CommandParser::new(),
            uptime_start: 0, // Will be set when initialized
        }
    }

    /// Initialize the system command handler
    pub async fn initialize(&mut self) -> Result<(), &'static str> {
        // Set uptime start time (in a real implementation, this would use embassy_time)
        self.uptime_start = 0; // Placeholder
        Ok(())
    }

    /// Process a system-related command and generate a response
    pub async fn process_system_command<T: UsbCdc>(
        &mut self,
        command: UsbCommand,
        sensor_count: u8,
        sensor_ready: bool,
        terminal: &Terminal<T>,
    ) -> UsbResponse {
        match command {
            UsbCommand::GetDebugInfo => self.handle_debug_info(sensor_count, terminal).await,

            UsbCommand::GetStatus => self.handle_status(sensor_ready).await,

            UsbCommand::Ping => self.handle_ping().await,

            UsbCommand::Help => self.handle_help().await,

            UsbCommand::Unknown(cmd) => self.handle_unknown_command(cmd).await,

            _ => {
                // This handler only processes system commands
                let mut error_msg = String::new();
                let _ = error_msg.push_str("Invalid system command");
                UsbResponse::Error(error_msg)
            }
        }
    }

    /// Handle GetDebugInfo command
    async fn handle_debug_info<T: UsbCdc>(&self, sensor_count: u8, terminal: &Terminal<T>) -> UsbResponse {
        let debug_info = DebugInfo {
            uptime_ms: 0,   // Would calculate actual uptime
            free_memory: 0, // Would get actual free memory
            usb_connected: terminal.is_connected(),
            sensor_count,
        };
        UsbResponse::DebugInfo(debug_info)
    }

    /// Handle GetStatus command
    async fn handle_status(&self, sensor_ready: bool) -> UsbResponse {
        let status = DeviceStatus {
            is_ready: true,
            sensor_initialized: sensor_ready,
            last_error: None,
        };
        UsbResponse::Status(status)
    }

    /// Handle Ping command
    async fn handle_ping(&self) -> UsbResponse {
        UsbResponse::Ack
    }

    /// Handle Help command
    async fn handle_help(&self) -> UsbResponse {
        let help_text = self.parser.get_help_text();
        UsbResponse::Help(help_text)
    }

    /// Handle Unknown command
    async fn handle_unknown_command(&self, cmd: String<64>) -> UsbResponse {
        let mut error_msg = String::new();
        let _ = error_msg.push_str("Unknown command: ");
        let _ = error_msg.push_str(&cmd);
        UsbResponse::Error(error_msg)
    }

    /// Get uptime in milliseconds
    pub fn get_uptime_ms(&self) -> u64 {
        // In real implementation, calculate actual uptime
        0
    }

    /// Get free memory (placeholder)
    pub fn get_free_memory(&self) -> u32 {
        // In real implementation, get actual free memory
        0
    }
}
