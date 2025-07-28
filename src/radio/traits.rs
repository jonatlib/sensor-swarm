// Radio communication traits
// This module defines generic, hardware-agnostic traits for radio communication

use defmt::Format;
use super::protocol::Packet;

/// Error types for radio communication operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum RadioError {
    /// Hardware initialization failed
    InitializationFailed,
    /// Transmission failed due to hardware error
    TransmissionFailed,
    /// Reception failed due to hardware error
    ReceptionFailed,
    /// Radio module is not ready for operation
    NotReady,
    /// Invalid packet format or size
    InvalidPacket,
    /// Timeout occurred during operation
    Timeout,
    /// Radio module is busy with another operation
    Busy,
    /// Signal quality too poor for reliable communication
    PoorSignalQuality,
    /// Buffer overflow or underflow
    BufferError,
    /// Generic hardware error
    HardwareError,
}

/// Generic trait for radio transmission functionality
/// 
/// This trait abstracts the hardware-specific details of radio transmission,
/// allowing the application logic to remain hardware-agnostic.
pub trait RadioTransmitter {
    /// Transmit a packet over the radio
    /// 
    /// # Arguments
    /// * `packet` - The packet to transmit
    /// 
    /// # Returns
    /// * `Ok(())` if transmission was successful
    /// * `Err(RadioError)` if transmission failed
    /// 
    /// # Notes
    /// This method should be non-blocking and async to maintain power efficiency.
    /// The implementation should handle all hardware-specific details including
    /// modulation, timing, and power management.
    async fn transmit(&mut self, packet: &Packet) -> Result<(), RadioError>;

    /// Check if the transmitter is ready for operation
    /// 
    /// # Returns
    /// * `true` if the transmitter is ready
    /// * `false` if the transmitter is busy or not initialized
    fn is_ready(&self) -> bool;

    /// Set the transmission power level
    /// 
    /// # Arguments
    /// * `power_level` - Power level from 0 (minimum) to 255 (maximum)
    /// 
    /// # Returns
    /// * `Ok(())` if power level was set successfully
    /// * `Err(RadioError)` if setting failed
    async fn set_power_level(&mut self, power_level: u8) -> Result<(), RadioError>;

    /// Get the current transmission power level
    /// 
    /// # Returns
    /// * Current power level from 0 to 255
    fn get_power_level(&self) -> u8;
}

/// Generic trait for radio reception functionality
/// 
/// This trait abstracts the hardware-specific details of radio reception,
/// allowing the application logic to remain hardware-agnostic.
pub trait RadioReceiver {
    /// Receive a packet from the radio
    /// 
    /// # Returns
    /// * `Ok(packet)` if a valid packet was received
    /// * `Err(RadioError)` if reception failed or no packet available
    /// 
    /// # Notes
    /// This method should be non-blocking and async. It should return immediately
    /// if no packet is available, allowing the caller to handle timeouts and
    /// other async operations efficiently.
    async fn receive(&mut self) -> Result<Packet, RadioError>;

    /// Check if a packet is available for reception
    /// 
    /// # Returns
    /// * `true` if a packet is ready to be received
    /// * `false` if no packet is available
    fn packet_available(&self) -> bool;

    /// Enable or disable the receiver
    /// 
    /// # Arguments
    /// * `enabled` - true to enable receiver, false to disable for power saving
    /// 
    /// # Returns
    /// * `Ok(())` if operation was successful
    /// * `Err(RadioError)` if operation failed
    async fn set_enabled(&mut self, enabled: bool) -> Result<(), RadioError>;

    /// Check if the receiver is currently enabled
    /// 
    /// # Returns
    /// * `true` if receiver is enabled
    /// * `false` if receiver is disabled
    fn is_enabled(&self) -> bool;

    /// Get the signal strength of the last received packet
    /// 
    /// # Returns
    /// * Signal strength in dBm, or None if no packet has been received
    fn get_rssi(&self) -> Option<i16>;
}

/// Combined trait for full-duplex radio communication
/// 
/// This trait combines both transmission and reception capabilities
/// for radios that support both operations.
pub trait RadioTransceiver: RadioTransmitter + RadioReceiver {
    /// Initialize the radio hardware
    /// 
    /// # Returns
    /// * `Ok(())` if initialization was successful
    /// * `Err(RadioError)` if initialization failed
    async fn initialize(&mut self) -> Result<(), RadioError>;

    /// Put the radio into low-power sleep mode
    /// 
    /// # Returns
    /// * `Ok(())` if sleep mode was entered successfully
    /// * `Err(RadioError)` if operation failed
    async fn sleep(&mut self) -> Result<(), RadioError>;

    /// Wake the radio from sleep mode
    /// 
    /// # Returns
    /// * `Ok(())` if wake operation was successful
    /// * `Err(RadioError)` if operation failed
    async fn wake(&mut self) -> Result<(), RadioError>;

    /// Get the current operating frequency in Hz
    /// 
    /// # Returns
    /// * Current frequency in Hz
    fn get_frequency(&self) -> u32;

    /// Set the operating frequency
    /// 
    /// # Arguments
    /// * `frequency_hz` - Frequency in Hz
    /// 
    /// # Returns
    /// * `Ok(())` if frequency was set successfully
    /// * `Err(RadioError)` if frequency setting failed
    async fn set_frequency(&mut self, frequency_hz: u32) -> Result<(), RadioError>;
}