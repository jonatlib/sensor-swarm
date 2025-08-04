/// USB Command Module - DISABLED
///
/// This module provides command definitions and parsing but USB functionality is disabled.
/// Command structure is kept for future use:
/// - Reading sensor data
/// - Getting debug information
/// - Extensible command system for future functionality
///
/// USB communication has been removed as requested, but the module structure remains.
///
/// The module is split into submodules for better organization:
/// - parser: Command parsing and supported command definitions
/// - responses: Response types and formatting
/// - sensor_commands: Sensor-related command handlers
/// - system_commands: System-related command handlers
use crate::hw::traits::DeviceManagement;
use crate::sensors::traits::EnvironmentalSensor;
use crate::terminal::Terminal;
use crate::usb::UsbCdc;
use defmt::*;
use heapless::Vec;

// Submodule declarations
pub mod parser;
pub mod responses;
pub mod sensor_commands;
pub mod system_commands;

// Re-export commonly used types
pub use parser::{CommandParser, SensorType, UsbCommand};
pub use responses::{DebugInfo, DeviceStatus, ResponseFormatter, UsbResponse};
pub use sensor_commands::SensorCommandHandler;
pub use system_commands::SystemCommandHandler;

/// Command terminator
const COMMAND_TERMINATOR: u8 = b'\n';

/// USB Command Handler
///
/// This struct manages USB command processing and response generation.
/// It's designed to be hardware-agnostic and work with any Terminal implementation.
/// It coordinates between different command handlers for better modularity.
pub struct UsbCommandHandler<T, S, D>
where
    T: UsbCdc,
    S: EnvironmentalSensor,
    D: DeviceManagement,
{
    terminal: Terminal<T>,
    device_manager: D,
    parser: CommandParser,
    sensor_handler: SensorCommandHandler<S>,
    system_handler: SystemCommandHandler,
    response_formatter: ResponseFormatter,
    command_buffer: Vec<u8, 256>,
    response_buffer: Vec<u8, 512>,
}

impl<T, S, D> UsbCommandHandler<T, S, D>
where
    T: UsbCdc,
    S: EnvironmentalSensor,
    D: DeviceManagement,
{
    /// Create a new USB command handler
    pub fn new(terminal: Terminal<T>, device_manager: D) -> Self {
        Self {
            system_handler: SystemCommandHandler::new(),
            terminal,
            device_manager,
            parser: CommandParser::new(),
            sensor_handler: SensorCommandHandler::new(),
            response_formatter: ResponseFormatter::new(),
            command_buffer: Vec::new(),
            response_buffer: Vec::new(),
        }
    }

    /// Set the sensor instance for the command handler
    pub fn set_sensor(&mut self, sensor: S) {
        self.sensor_handler.set_sensor(sensor);
    }

    /// Initialize the command handler
    pub async fn initialize(&mut self) -> Result<(), &'static str> {
        // Initialize system handler
        self.system_handler.initialize().await?;

        info!("USB Command Handler initialized");
        Ok(())
    }

    /// Main command processing loop
    /// This should be called as an embassy task
    pub async fn run(&mut self) -> Result<(), &'static str> {
        loop {
            // Check if USB is connected
            if !self.terminal.is_connected() {
                // Wait a bit before checking again
                embassy_time::Timer::after_millis(100).await;
                continue;
            }

            // Try to receive a command
            match self.receive_command().await {
                Ok(Some(command)) => {
                    let response = self.process_command(command).await;
                    if let Err(e) = self.send_response(response).await {
                        warn!("Failed to send response: {}", e);
                    }
                }
                Ok(None) => {
                    // No command received, continue
                }
                Err(e) => {
                    warn!("Error receiving command: {}", e);
                }
            }
        }
    }

    /// Receive and parse a command from USB
    async fn receive_command(&mut self) -> Result<Option<UsbCommand>, &'static str> {
        // Clear the command buffer
        self.command_buffer.clear();

        // Read bytes until we get a complete command (terminated by newline)
        let mut temp_buffer = [0u8; 32];
        loop {
            match self.terminal.read_bytes(&mut temp_buffer).await {
                Ok(0) => return Ok(None), // No data available
                Ok(bytes_read) => {
                    for &byte in &temp_buffer[..bytes_read] {
                        if byte == COMMAND_TERMINATOR {
                            // Command complete, parse it
                            return Ok(Some(self.parser.parse_command(&self.command_buffer)));
                        } else if self.command_buffer.len() < self.command_buffer.capacity() - 1 {
                            let _ = self.command_buffer.push(byte);
                        } else {
                            // Command too long
                            let mut error_msg = heapless::String::new();
                            let _ = error_msg.push_str("COMMAND_TOO_LONG");
                            return Ok(Some(UsbCommand::Unknown(error_msg)));
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Process a command and generate a response
    async fn process_command(&mut self, command: UsbCommand) -> UsbResponse {
        match command {
            // Sensor commands
            UsbCommand::ReadSensors | UsbCommand::ReadSensorType(_) => {
                self.sensor_handler.process_sensor_command(command).await
            }

            // System commands
            UsbCommand::GetDebugInfo
            | UsbCommand::GetStatus
            | UsbCommand::Ping
            | UsbCommand::Help
            | UsbCommand::Unknown(_) => {
                let sensor_count = self.sensor_handler.sensor_count();
                let sensor_ready = self.sensor_handler.is_sensor_ready();
                self.system_handler
                    .process_system_command(command, sensor_count, sensor_ready, &self.terminal)
                    .await
            }

            // Reboot commands - these need special handling as they don't return
            UsbCommand::RebootCpu => {
                // Send acknowledgment before rebooting
                let ack_response = UsbResponse::Ack;
                if let Err(e) = self.send_response(ack_response).await {
                    warn!("Failed to send reboot acknowledgment: {}", e);
                }

                // Perform the reboot - this will not return
                info!("Executing CPU reboot command");
                self.device_manager.reboot();
            }

            UsbCommand::RebootCpuToDfu => {
                // Send acknowledgment before rebooting
                let ack_response = UsbResponse::Ack;
                if let Err(e) = self.send_response(ack_response).await {
                    warn!("Failed to send DFU reboot acknowledgment: {}", e);
                }

                // Perform the DFU reboot - this will not return
                info!("Executing CPU reboot to DFU mode command");
                self.device_manager.reboot_to_bootloader();
            }
        }
    }

    /// Send a response over USB
    async fn send_response(&mut self, response: UsbResponse) -> Result<(), &'static str> {
        // Format the response
        let response_text = self.response_formatter.format_response(response);

        // Convert to bytes and send
        let response_bytes = response_text.as_bytes();
        self.terminal.write_bytes(response_bytes).await?;

        // Send terminator
        self.terminal.write_bytes(&[COMMAND_TERMINATOR]).await?;

        Ok(())
    }
}

/// Convenience function to create and run a USB command handler task
/// This can be used in the main application to easily set up USB command handling
pub async fn run_usb_command_handler<T, S, D>(
    terminal: Terminal<T>,
    device_manager: D,
    sensor: Option<S>,
) -> Result<(), &'static str>
where
    T: UsbCdc,
    S: EnvironmentalSensor,
    D: DeviceManagement,
{
    let mut handler = UsbCommandHandler::new(terminal, device_manager);

    if let Some(sensor) = sensor {
        handler.set_sensor(sensor);
    }

    handler.initialize().await?;
    handler.run().await
}
