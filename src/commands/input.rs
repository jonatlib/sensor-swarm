/// Terminal input handling and buffering module
/// This module handles reading data from terminal and buffering until ENTER key

use crate::terminal::SharedTerminal;
use crate::usb::UsbCdc;
use heapless::{String, Vec};

/// Command buffer size for incoming commands
const COMMAND_BUFFER_SIZE: usize = 256;

/// Input handler that manages terminal input buffering
pub struct InputHandler<T: UsbCdc> {
    terminal: SharedTerminal<T>,
    command_buffer: Vec<u8, COMMAND_BUFFER_SIZE>,
}

impl<T: UsbCdc> InputHandler<T> {
    /// Create a new input handler with the given shared terminal
    pub fn new(terminal: SharedTerminal<T>) -> Self {
        Self {
            terminal,
            command_buffer: Vec::new(),
        }
    }

    /// Main input handling loop - reads and buffers terminal input
    /// Returns complete command strings when ENTER is detected
    pub async fn read_command(&mut self) -> Result<Option<String<COMMAND_BUFFER_SIZE>>, &'static str> {
        let mut temp_buffer = [0u8; 32];

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
                    // Terminal disconnected
                    return Err("Terminal disconnected");
                }
            }
        };

        // Process received bytes
        if bytes_read > 0 {
            for &byte in &temp_buffer[..bytes_read] {
                match byte {
                    b'\n' | b'\r' => {
                        // ENTER key detected - return complete command
                        if !self.command_buffer.is_empty() {
                            let command_str = match core::str::from_utf8(&self.command_buffer) {
                                Ok(s) => {
                                    let mut result = heapless::String::new();
                                    let _ = result.push_str(s.trim());
                                    result
                                }
                                Err(_) => {
                                    self.command_buffer.clear();
                                    return Err("Invalid UTF-8 in command");
                                }
                            };
                            self.command_buffer.clear();
                            return Ok(Some(command_str));
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

        Ok(None) // No complete command yet
    }

    /// Send response back to terminal
    pub async fn send_response(&mut self, response: &str) -> Result<(), &'static str> {
        let mut terminal = self.terminal.lock().await;
        terminal.write_logs(response).await
    }
}