// Sensor traits and data structures
// This module defines generic traits for environmental sensors

use bitfield_struct::bitfield;
use defmt::Format;

/// Error types for sensor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum SensorError {
    /// Sensor initialization failed
    InitializationFailed,
    /// Communication with sensor failed
    CommunicationFailed,
    /// Sensor reading is out of valid range
    OutOfRange,
    /// Sensor is not ready for operation
    NotReady,
    /// Timeout occurred during sensor operation
    Timeout,
    /// Sensor calibration failed or is invalid
    CalibrationError,
    /// Sensor hardware malfunction detected
    HardwareFault,
    /// Invalid sensor configuration
    InvalidConfiguration,
    /// Sensor data is corrupted or invalid
    DataCorruption,
    /// Generic sensor error
    GenericError,
}

/// Environmental sensor data structure
///
/// This structure contains the readings from environmental sensors
/// such as temperature, humidity, pressure, etc.
#[derive(Debug, Clone, Copy, PartialEq, Format)]
pub struct EnvironmentalData {
    /// Temperature in degrees Celsius (multiplied by 100 for precision)
    /// Example: 2550 represents 25.50°C
    pub temperature_celsius_x100: i32,

    /// Relative humidity as percentage (multiplied by 100 for precision)
    /// Example: 6525 represents 65.25% RH
    pub humidity_percent_x100: u32,

    /// Atmospheric pressure in Pascals
    /// Example: 101325 represents standard atmospheric pressure
    pub pressure_pa: u32,

    /// Light intensity in lux (multiplied by 10 for precision)
    /// Example: 1500 represents 150.0 lux
    pub light_lux_x10: u32,

    /// Timestamp of the reading in milliseconds since system start
    pub timestamp_ms: u64,

    /// Validity flags indicating which readings are valid
    pub validity: DataValidity,
}

/// Validity flags for sensor data
#[bitfield(u8)]
#[derive(PartialEq, Eq, Format)]
pub struct DataValidity {
    /// Temperature reading is valid
    pub temperature_valid: bool,
    /// Humidity reading is valid
    pub humidity_valid: bool,
    /// Pressure reading is valid
    pub pressure_valid: bool,
    /// Light reading is valid
    pub light_valid: bool,
    /// Reserved bits (unused)
    #[bits(4)]
    _reserved: u8,
}

impl DataValidity {
    /// Create a DataValidity with all fields set to true
    pub const fn all_valid() -> Self {
        Self::new()
            .with_temperature_valid(true)
            .with_humidity_valid(true)
            .with_pressure_valid(true)
            .with_light_valid(true)
    }

    /// Check if any sensor data is valid
    pub fn has_valid_data(&self) -> bool {
        self.temperature_valid()
            || self.humidity_valid()
            || self.pressure_valid()
            || self.light_valid()
    }

    /// Check if all sensor data is valid
    pub fn all_data_valid(&self) -> bool {
        self.temperature_valid()
            && self.humidity_valid()
            && self.pressure_valid()
            && self.light_valid()
    }
}

impl EnvironmentalData {
    /// Create a new EnvironmentalData with default values
    pub fn new() -> Self {
        Self {
            temperature_celsius_x100: 0,
            humidity_percent_x100: 0,
            pressure_pa: 0,
            light_lux_x10: 0,
            timestamp_ms: 0,
            validity: DataValidity::new(),
        }
    }

    /// Get temperature in degrees Celsius as a floating-point value
    pub fn temperature_celsius(&self) -> f32 {
        self.temperature_celsius_x100 as f32 / 100.0
    }

    /// Get humidity as a percentage floating-point value
    pub fn humidity_percent(&self) -> f32 {
        self.humidity_percent_x100 as f32 / 100.0
    }

    /// Get light intensity in lux as a floating-point value
    pub fn light_lux(&self) -> f32 {
        self.light_lux_x10 as f32 / 10.0
    }

    /// Set temperature from floating-point Celsius value
    pub fn set_temperature_celsius(&mut self, temp_c: f32) {
        self.temperature_celsius_x100 = (temp_c * 100.0) as i32;
        self.validity = self.validity.with_temperature_valid(true);
    }

    /// Set humidity from floating-point percentage value
    pub fn set_humidity_percent(&mut self, humidity: f32) {
        self.humidity_percent_x100 = (humidity * 100.0) as u32;
        self.validity = self.validity.with_humidity_valid(true);
    }

    /// Set light intensity from floating-point lux value
    pub fn set_light_lux(&mut self, lux: f32) {
        self.light_lux_x10 = (lux * 10.0) as u32;
        self.validity = self.validity.with_light_valid(true);
    }

    /// Set pressure in Pascals
    pub fn set_pressure_pa(&mut self, pressure: u32) {
        self.pressure_pa = pressure;
        self.validity = self.validity.with_pressure_valid(true);
    }
}

impl Default for EnvironmentalData {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic trait for environmental sensors
///
/// This trait provides a hardware-agnostic interface for reading
/// environmental data from various sensor types.
pub trait EnvironmentalSensor {
    /// Read environmental data from the sensor
    ///
    /// # Returns
    /// * `Ok(EnvironmentalData)` containing the sensor readings
    /// * `Err(SensorError)` if reading failed
    ///
    /// # Notes
    /// This method should be async and non-blocking to maintain power efficiency.
    /// The implementation should handle all hardware-specific communication
    /// protocols and data conversion.
    fn read(
        &mut self,
    ) -> impl core::future::Future<Output = Result<EnvironmentalData, SensorError>> + Send;

    /// Initialize the sensor hardware
    ///
    /// # Returns
    /// * `Ok(())` if initialization was successful
    /// * `Err(SensorError)` if initialization failed
    fn initialize(&mut self) -> impl core::future::Future<Output = Result<(), SensorError>> + Send;

    /// Check if the sensor is ready for operation
    ///
    /// # Returns
    /// * `true` if the sensor is ready to provide readings
    /// * `false` if the sensor is not initialized or has an error
    fn is_ready(&self) -> bool;

    /// Put the sensor into low-power sleep mode
    ///
    /// # Returns
    /// * `Ok(())` if sleep mode was entered successfully
    /// * `Err(SensorError)` if operation failed
    fn sleep(&mut self) -> impl core::future::Future<Output = Result<(), SensorError>> + Send;

    /// Wake the sensor from sleep mode
    ///
    /// # Returns
    /// * `Ok(())` if wake operation was successful
    /// * `Err(SensorError)` if operation failed
    fn wake(&mut self) -> impl core::future::Future<Output = Result<(), SensorError>> + Send;

    /// Get the sensor's capabilities
    ///
    /// # Returns
    /// * `DataValidity` indicating which types of data this sensor can provide
    fn get_capabilities(&self) -> DataValidity;

    /// Perform sensor self-test
    ///
    /// # Returns
    /// * `Ok(())` if self-test passed
    /// * `Err(SensorError)` if self-test failed
    fn self_test(&mut self) -> impl core::future::Future<Output = Result<(), SensorError>> + Send;

    /// Get the minimum time between readings in milliseconds
    ///
    /// # Returns
    /// * Minimum interval between sensor readings in milliseconds
    fn get_min_reading_interval_ms(&self) -> u32;
}
