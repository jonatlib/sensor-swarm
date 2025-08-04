/// Hardware-independent command handling module
/// This module provides command parsing and handling using the Terminal
/// Commands are read from terminal, parsed when ENTER is detected, and responses sent back

use crate::terminal::SharedTerminal;
use crate::usb::UsbCdc;
use heapless::{String, Vec};

/// Command buffer size for incoming commands
const COMMAND_BUFFER_SIZE: usize = 256;

/// Command handler that works with any Terminal implementation
pub struct CommandHandler<T: UsbCdc> {
    terminal: SharedTerminal<T>,
    command_buffer: Vec<u8, COMMAND_BUFFER_SIZE>,
}

impl<T: UsbCdc> CommandHandler<T> {
    /// Create a new command handler with the given shared terminal
    pub fn new(terminal: SharedTerminal<T>) -> Self {
        Self {
            terminal,
            command_buffer: Vec::new(),
        }
    }

    /// Main command handling loop
    /// Reads bytes from terminal, buffers them, and processes commands on ENTER
    pub async fn run(&mut self) -> Result<(), &'static str> {
        let mut temp_buffer = [0u8; 32];

        loop {
            // Wait for terminal connection
            {
                let mut terminal = self.terminal.lock().await;
                if !terminal.is_connected() {
                    terminal.wait_connection().await;
                    let _ = terminal.write_logs("Command handler ready - type 'help' for available commands").await;
                }
            }

            // Read bytes from terminal (non-blocking)
            let bytes_read = {
                let mut terminal = self.terminal.lock().await;
                match terminal.read_bytes(&mut temp_buffer).await {
                    Ok(count) => count,
                    Err(_) => {
                        // Terminal disconnected, continue loop to wait for reconnection
                        continue;
                    }
                }
            };

            // Process received bytes
            if bytes_read > 0 {
                for &byte in &temp_buffer[..bytes_read] {
                    match byte {
                        b'\n' | b'\r' => {
                            // ENTER key detected - process command
                            if !self.command_buffer.is_empty() {
                                self.process_command().await;
                                self.command_buffer.clear();
                            }
                        }
                        b'\x08' | b'\x7f' => {
                            // Backspace - remove last character
                            if !self.command_buffer.is_empty() {
                                self.command_buffer.pop();
                                // Echo backspace to terminal
                                let mut terminal = self.terminal.lock().await;
                                let _ = terminal.write_bytes(b"\x08 \x08").await;
                            }
                        }
                        32..=126 => {
                            // Printable ASCII character
                            if self.command_buffer.len() < COMMAND_BUFFER_SIZE - 1 {
                                if self.command_buffer.push(byte).is_ok() {
                                    // Echo character back to terminal
                                    let mut terminal = self.terminal.lock().await;
                                    let _ = terminal.write_bytes(&[byte]).await;
                                }
                            }
                        }
                        _ => {
                            // Ignore other characters
                        }
                    }
                }
            }

            // Small delay to prevent busy waiting
            embassy_time::Timer::after_millis(10).await;
        }
    }

    /// Process a complete command from the buffer
    async fn process_command(&mut self) {
        // Convert buffer to string
        let command_str = match core::str::from_utf8(&self.command_buffer) {
            Ok(s) => s.trim(),
            Err(_) => {
                let mut terminal = self.terminal.lock().await;
                let _ = terminal.write_logs("Error: Invalid UTF-8 in command").await;
                return;
            }
        };

        // Parse and execute command
        let response = self.parse_and_execute_command(command_str).await;

        // Send response to terminal
        let mut terminal = self.terminal.lock().await;
        let _ = terminal.write_logs(&response).await;
    }

    /// Parse command string and execute appropriate action
    async fn parse_and_execute_command(&self, command: &str) -> String<512> {
        let mut response = String::new();

        // Split command into parts
        let parts: Vec<&str, 8> = command.split_whitespace().collect();
        
        if parts.is_empty() {
            let _ = response.push_str("Error: Empty command");
            return response;
        }

        match parts[0] {
            "help" => {
                let _ = response.push_str("Available commands:\n");
                let _ = response.push_str("  help - Show this help message\n");
                let _ = response.push_str("  status - Show device status\n");
                let _ = response.push_str("  reboot - Reboot the device\n");
                let _ = response.push_str("  version - Show firmware version");
            }
            "status" => {
                let _ = response.push_str("Device Status:\n");
                let _ = response.push_str("  USB: Connected\n");
                let _ = response.push_str("  Terminal: Active\n");
                let _ = response.push_str("  System: Running");
            }
            "version" => {
                let _ = response.push_str("Sensor Swarm Firmware v1.0.0\n");
                let _ = response.push_str("Built with new modular architecture");
            }
            "reboot" => {
                let _ = response.push_str("Rebooting device...");
                // Note: Actual reboot would be implemented here using device manager
            }
            _ => {
                let _ = core::fmt::write(&mut response, format_args!("Error: Unknown command '{}'. Type 'help' for available commands.", parts[0]));
            }
        }

        response
    }
}

/// Create and run a command handler task
/// This is a convenience function for spawning the command handler
pub async fn run_command_handler<T: UsbCdc>(terminal: SharedTerminal<T>) -> Result<(), &'static str> {
    let mut handler = CommandHandler::new(terminal);
    handler.run().await
}