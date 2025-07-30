/// System Commands Handler Module
/// 
/// This module handles all system-related USB commands including debug info,
/// device status, ping, and help commands. It provides system-level information
/// and utility functions.

use crate::hw::traits::UsbCommunication;
use crate::usb_commands::parser::{UsbCommand, CommandParser};
use crate::usb_commands::responses::{UsbResponse, DebugInfo, DeviceStatus};
use heapless::String;

/// System Commands Handler
/// 
/// Handles processing of system-related commands and generates appropriate responses.
pub struct SystemCommandHandler<U> 
where
    U: UsbCommunication,
{
    usb_manager: U,
    parser: CommandParser,
    uptime_start: u64, // For tracking uptime
}

impl<U> SystemCommandHandler<U>
where
    U: UsbCommunication,
{
    /// Create a new system command handler
    pub fn new(usb_manager: U) -> Self {
        Self {
            usb_manager,
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
    pub async fn process_system_command(&mut self, command: UsbCommand, sensor_count: u8, sensor_ready: bool) -> UsbResponse {
        match command {
            UsbCommand::GetDebugInfo => {
                self.handle_debug_info(sensor_count).await
            }
            
            UsbCommand::GetStatus => {
                self.handle_status(sensor_ready).await
            }
            
            UsbCommand::Ping => {
                self.handle_ping().await
            }
            
            UsbCommand::Help => {
                self.handle_help().await
            }
            
            UsbCommand::Unknown(cmd) => {
                self.handle_unknown_command(cmd).await
            }
            
            _ => {
                // This handler only processes system commands
                let mut error_msg = String::new();
                let _ = error_msg.push_str("Invalid system command");
                UsbResponse::Error(error_msg)
            }
        }
    }

    /// Handle GetDebugInfo command
    async fn handle_debug_info(&self, sensor_count: u8) -> UsbResponse {
        let debug_info = DebugInfo {
            uptime_ms: 0, // Would calculate actual uptime
            free_memory: 0, // Would get actual free memory
            usb_connected: self.usb_manager.is_connected(),
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

    /// Check if USB is connected
    pub fn is_usb_connected(&self) -> bool {
        self.usb_manager.is_connected()
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

/// Utility functions for system information
impl<U> SystemCommandHandler<U>
where
    U: UsbCommunication,
{
    /// Create a comprehensive system status
    pub fn create_system_status(&self, sensor_count: u8, sensor_ready: bool) -> DeviceStatus {
        DeviceStatus {
            is_ready: self.usb_manager.is_connected(),
            sensor_initialized: sensor_ready,
            last_error: None,
        }
    }

    /// Create comprehensive debug information
    pub fn create_debug_info(&self, sensor_count: u8) -> DebugInfo {
        DebugInfo {
            uptime_ms: self.get_uptime_ms(),
            free_memory: self.get_free_memory(),
            usb_connected: self.usb_manager.is_connected(),
            sensor_count,
        }
    }
}