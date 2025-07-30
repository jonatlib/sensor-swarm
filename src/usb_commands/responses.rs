/// Response Types Module
///
/// This module defines all response types and data structures used by USB commands.
/// It provides a centralized location for response formatting and serialization.
use crate::sensors::traits::EnvironmentalData;
use heapless::String;

/// Maximum response length in bytes  
pub const MAX_RESPONSE_LENGTH: usize = 512;

/// Response structure for USB commands
#[derive(Debug, Clone)]
pub enum UsbResponse {
    /// Sensor data response
    SensorData(EnvironmentalData),
    /// Debug information response
    DebugInfo(DebugInfo),
    /// Device status response
    Status(DeviceStatus),
    /// Simple acknowledgment
    Ack,
    /// Error response
    Error(String<128>),
    /// Help text response
    Help(String<256>),
}

/// Debug information structure
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub uptime_ms: u64,
    pub free_memory: u32,
    pub usb_connected: bool,
    pub sensor_count: u8,
}

/// Device status information
#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub is_ready: bool,
    pub sensor_initialized: bool,
    pub last_error: Option<String<64>>,
}

/// Response Formatter
///
/// Handles formatting of responses into strings for transmission over USB.
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Create a new response formatter
    pub fn new() -> Self {
        Self
    }

    /// Format a response into a string
    pub fn format_response(&self, response: UsbResponse) -> String<MAX_RESPONSE_LENGTH> {
        let mut formatted = String::new();

        match response {
            UsbResponse::SensorData(_data) => {
                let _ = formatted.push_str("SENSOR_DATA:");
                // Format sensor data (simplified for now)
                let _ = formatted.push_str("temp=");
                // Would format actual values here
                let _ = formatted.push_str("25.0");
            }

            UsbResponse::DebugInfo(_info) => {
                let _ = formatted.push_str("DEBUG_INFO:");
                let _ = formatted.push_str("uptime=");
                // Would format actual values here
            }

            UsbResponse::Status(status) => {
                let _ = formatted.push_str("STATUS:");
                if status.is_ready {
                    let _ = formatted.push_str("READY");
                } else {
                    let _ = formatted.push_str("NOT_READY");
                }
            }

            UsbResponse::Ack => {
                let _ = formatted.push_str("ACK");
            }

            UsbResponse::Error(error) => {
                let _ = formatted.push_str("ERROR:");
                let _ = formatted.push_str(&error);
            }

            UsbResponse::Help(help) => {
                let _ = formatted.push_str(&help);
            }
        }

        formatted
    }
}

impl Default for ResponseFormatter {
    fn default() -> Self {
        Self::new()
    }
}
