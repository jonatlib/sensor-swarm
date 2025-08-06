/// Hardware-independent command handling module
/// This module coordinates the separate command sub-modules:
/// - input: handles terminal input buffering
/// - parser: parses commands into enums
/// - executor: executes commands and generates responses

use crate::terminal::SharedTerminal;
use crate::usb::UsbCdc;
use crate::hw::traits::DeviceManagement;

// Sub-modules
pub mod input;
pub mod parser;
pub mod response;
pub mod executor;

// Re-export public types from sub-modules
pub use parser::{Command, SensorType};
pub use input::InputHandler;
pub use parser::CommandParser;
pub use response::{Response, SensorValue};
pub use executor::CommandExecutor;

/// Main command handler that coordinates all sub-modules
pub struct CommandHandler<T: UsbCdc, D: for<'d> DeviceManagement<'d>> {
    input_handler: InputHandler<T>,
    parser: CommandParser,
    executor: CommandExecutor<D>,
}

impl<T: UsbCdc, D: for<'d> DeviceManagement<'d>> CommandHandler<T, D> {
    /// Create a new command handler with the given shared terminal and device manager
    pub fn new(terminal: SharedTerminal<T>, device_manager: D) -> Self {
        Self {
            input_handler: InputHandler::new(terminal),
            parser: CommandParser::new(),
            executor: CommandExecutor::new(device_manager),
        }
    }

    /// Main command handling loop
    /// Coordinates input reading, parsing, and command execution
    pub async fn run(&mut self) -> Result<(), &'static str> {
        loop {
            // Read command from input handler
            match self.input_handler.read_command().await {
                Ok(Some(command_str)) => {
                    // Parse the command
                    let command = self.parser.parse(command_str.as_str());
                    
                    // Execute the command
                    let response = self.executor.execute(command).await;
                    
                    // Convert response to string and send back through input handler
                    let response_str = self.executor.response_to_string(&response);
                    let _ = self.input_handler.send_response(response_str.as_str()).await;
                }
                Ok(None) => {
                    // No complete command yet, continue reading
                }
                Err(e) => {
                    // Handle error by sending error message
                    let _ = self.input_handler.send_response(e).await;
                }
            }

            // Small delay to prevent busy waiting
            embassy_time::Timer::after_millis(10).await;
        }
    }

    /// Parse command string into Command enum (for backward compatibility)
    pub fn parse_command(&self, command_str: &str) -> Command {
        self.parser.parse(command_str)
    }
}

/// Create and run a command handler task
/// This is a convenience function for spawning the command handler
pub async fn run_command_handler<T: UsbCdc, D: for<'d> DeviceManagement<'d>>(terminal: SharedTerminal<T>, device_manager: D) -> Result<(), &'static str> {
    let mut handler = CommandHandler::new(terminal, device_manager);
    handler.run().await
}