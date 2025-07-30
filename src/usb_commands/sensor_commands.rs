/// Sensor Commands Handler Module
/// 
/// This module handles all sensor-related USB commands including reading sensor data
/// and filtering by sensor type. It provides a clean interface between the USB command
/// system and the sensor subsystem.

use crate::sensors::traits::{EnvironmentalSensor, EnvironmentalData};
use crate::usb_commands::parser::{UsbCommand, SensorType};
use crate::usb_commands::responses::UsbResponse;
use heapless::String;

/// Sensor Commands Handler
/// 
/// Handles processing of sensor-related commands and generates appropriate responses.
pub struct SensorCommandHandler<S> 
where
    S: EnvironmentalSensor,
{
    sensor: Option<S>,
}

impl<S> SensorCommandHandler<S>
where
    S: EnvironmentalSensor,
{
    /// Create a new sensor command handler
    pub fn new() -> Self {
        Self {
            sensor: None,
        }
    }

    /// Set the sensor instance for the command handler
    pub fn set_sensor(&mut self, sensor: S) {
        self.sensor = Some(sensor);
    }

    /// Process a sensor-related command and generate a response
    pub async fn process_sensor_command(&mut self, command: UsbCommand) -> UsbResponse {
        match command {
            UsbCommand::ReadSensors => {
                self.handle_read_sensors().await
            }
            
            UsbCommand::ReadSensorType(sensor_type) => {
                self.handle_read_sensor_type(sensor_type).await
            }
            
            _ => {
                // This handler only processes sensor commands
                let mut error_msg = String::new();
                let _ = error_msg.push_str("Invalid sensor command");
                UsbResponse::Error(error_msg)
            }
        }
    }

    /// Handle ReadSensors command
    async fn handle_read_sensors(&mut self) -> UsbResponse {
        if let Some(ref mut sensor) = self.sensor {
            match sensor.read().await {
                Ok(data) => UsbResponse::SensorData(data),
                Err(_e) => {
                    let mut error_msg = String::new();
                    let _ = error_msg.push_str("Sensor read failed: ");
                    // In real implementation, format the error properly
                    UsbResponse::Error(error_msg)
                }
            }
        } else {
            let mut error_msg = String::new();
            let _ = error_msg.push_str("No sensor available");
            UsbResponse::Error(error_msg)
        }
    }

    /// Handle ReadSensorType command
    async fn handle_read_sensor_type(&mut self, sensor_type: SensorType) -> UsbResponse {
        if let Some(ref mut sensor) = self.sensor {
            match sensor.read().await {
                Ok(data) => {
                    // Create a filtered response with only the requested sensor type
                    let mut filtered_data = EnvironmentalData::new();
                    match sensor_type {
                        SensorType::Temperature => {
                            filtered_data.set_temperature_celsius(data.temperature_celsius());
                        }
                        SensorType::Humidity => {
                            filtered_data.set_humidity_percent(data.humidity_percent());
                        }
                        SensorType::Light => {
                            filtered_data.set_light_lux(data.light_lux());
                        }
                        SensorType::Pressure => {
                            // Pressure getter not available in the trait, would need to be added
                            // For now, return full data
                        }
                    }
                    UsbResponse::SensorData(filtered_data)
                }
                Err(_) => {
                    let mut error_msg = String::new();
                    let _ = error_msg.push_str("Sensor read failed");
                    UsbResponse::Error(error_msg)
                }
            }
        } else {
            let mut error_msg = String::new();
            let _ = error_msg.push_str("No sensor available");
            UsbResponse::Error(error_msg)
        }
    }

    /// Check if sensor is available and ready
    pub fn is_sensor_ready(&self) -> bool {
        self.sensor.as_ref().map_or(false, |s| s.is_ready())
    }

    /// Get sensor count (0 or 1)
    pub fn sensor_count(&self) -> u8 {
        if self.sensor.is_some() { 1 } else { 0 }
    }
}

impl<S> Default for SensorCommandHandler<S>
where
    S: EnvironmentalSensor,
{
    fn default() -> Self {
        Self::new()
    }
}