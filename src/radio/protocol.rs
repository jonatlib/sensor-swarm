// Radio protocol definitions
// This module defines the data structures for our custom radio packet format

use bitfield_struct::bitfield;
#[cfg(feature = "defmt")]
use defmt::Format;

/// Maximum size of the packet payload in bytes
pub const MAX_PAYLOAD_SIZE: usize = 32;

/// Total packet size in bytes (header + payload)
pub const PACKET_SIZE_BYTES: usize = core::mem::size_of::<Header>() + MAX_PAYLOAD_SIZE;

/// Packet header containing routing and control information
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
#[repr(C)]
pub struct Header {
    /// Unique identifier of the sender node
    pub sender_id: u16,
    /// Target node identifier (0 for broadcast)
    pub target_id: u16,
    /// Sequence number for packet ordering and duplicate detection
    pub sequence_number: u16,
    /// Control flags and packet type information
    pub control: PacketControl,
    /// Length of the actual payload data
    pub payload_len: u8,
}

/// Control flags and packet type information
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PacketControl {
    /// Acknowledgment request flag
    pub ack_request: bool,
    /// Acknowledgment response flag  
    pub ack_response: bool,
    /// Emergency/priority flag
    pub emergency: bool,
    /// Retransmission flag
    pub retransmit: bool,
    /// Reserved bits (unused)
    #[bits(4)]
    _reserved: u8,
}

impl PacketControl {
    /// Check if acknowledgment is requested
    pub fn is_ack_request(&self) -> bool {
        self.ack_request()
    }

    /// Check if this is an acknowledgment response
    pub fn is_ack(&self) -> bool {
        self.ack_response()
    }

    /// Check if this is an emergency packet
    pub fn is_emergency(&self) -> bool {
        self.emergency()
    }

    /// Check if this is a retransmitted packet
    pub fn is_retransmit(&self) -> bool {
        self.retransmit()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for PacketControl {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "PacketControl {{ ack_request: {}, ack_response: {}, emergency: {}, retransmit: {} }}",
            self.ack_request(),
            self.ack_response(),
            self.emergency(),
            self.retransmit()
        )
    }
}

/// Complete radio packet structure
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct Packet {
    /// Packet header with routing and control information
    pub header: Header,
    /// Fixed-size payload array
    pub payload: [u8; MAX_PAYLOAD_SIZE],
}

impl Packet {
    /// Create a new packet with the given parameters
    pub fn new(sender_id: u16, target_id: u16, sequence_number: u16, payload: &[u8]) -> Self {
        let mut packet = Self {
            header: Header {
                sender_id,
                target_id,
                sequence_number,
                control: PacketControl::new(),
                payload_len: payload.len().min(MAX_PAYLOAD_SIZE) as u8,
            },
            payload: [0; MAX_PAYLOAD_SIZE],
        };

        // Copy payload data
        let copy_len = payload.len().min(MAX_PAYLOAD_SIZE);
        packet.payload[..copy_len].copy_from_slice(&payload[..copy_len]);

        packet
    }

    /// Get the actual payload data (excluding padding)
    pub fn payload_data(&self) -> &[u8] {
        &self.payload[..self.header.payload_len as usize]
    }

    /// Convert packet to byte array for transmission
    pub fn to_bytes(&self) -> [u8; PACKET_SIZE_BYTES] {
        let mut bytes = [0u8; PACKET_SIZE_BYTES];

        // Serialize header
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &self.header as *const Header as *const u8,
                core::mem::size_of::<Header>(),
            )
        };

        let header_size = core::mem::size_of::<Header>();
        bytes[..header_size].copy_from_slice(header_bytes);
        bytes[header_size..].copy_from_slice(&self.payload);

        bytes
    }

    /// Create packet from byte array received from radio
    pub fn from_bytes(bytes: &[u8; PACKET_SIZE_BYTES]) -> Self {
        let header_size = core::mem::size_of::<Header>();

        // Deserialize header
        let header = unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const Header) };

        let mut payload = [0u8; MAX_PAYLOAD_SIZE];
        payload.copy_from_slice(&bytes[header_size..]);

        Self { header, payload }
    }
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use super::*;

    #[test]
    fn test_packet_control_flags() {
        let mut control = PacketControl::new();

        // Test initial state
        defmt::assert!(!control.is_ack_request());
        defmt::assert!(!control.is_ack());
        defmt::assert!(!control.is_emergency());
        defmt::assert!(!control.is_retransmit());

        // Test setting flags
        control.set_ack_request(true);
        defmt::assert!(control.is_ack_request());

        control.set_ack_response(true);
        defmt::assert!(control.is_ack());

        control.set_emergency(true);
        defmt::assert!(control.is_emergency());

        control.set_retransmit(true);
        defmt::assert!(control.is_retransmit());

        // Test unsetting flags
        control.set_ack_request(false);
        defmt::assert!(!control.is_ack_request());
    }

    #[test]
    fn test_packet_creation() {
        let payload = b"Hello, World!";
        let packet = Packet::new(0x1234, 0x5678, 42, payload);

        defmt::assert!(packet.header.sender_id == 0x1234);
        defmt::assert!(packet.header.target_id == 0x5678);
        defmt::assert!(packet.header.sequence_number == 42);
        defmt::assert!(packet.header.payload_len == payload.len() as u8);
        defmt::assert!(packet.payload_data() == payload);
    }

    #[test]
    fn test_packet_creation_with_oversized_payload() {
        // Create payload larger than MAX_PAYLOAD_SIZE
        let large_payload = [0xAA; MAX_PAYLOAD_SIZE + 10];
        let packet = Packet::new(0x1111, 0x2222, 100, &large_payload);

        // Should truncate to MAX_PAYLOAD_SIZE
        defmt::assert!(packet.header.payload_len == MAX_PAYLOAD_SIZE as u8);
        defmt::assert!(packet.payload_data().len() == MAX_PAYLOAD_SIZE);

        // Check that the truncated data matches
        for i in 0..MAX_PAYLOAD_SIZE {
            defmt::assert!(packet.payload[i] == 0xAA);
        }
    }

    #[test]
    fn test_packet_creation_with_empty_payload() {
        let packet = Packet::new(0x0001, 0x0002, 1, &[]);

        defmt::assert!(packet.header.payload_len == 0);
        defmt::assert!(packet.payload_data().len() == 0);
    }

    #[test]
    fn test_packet_serialization_deserialization() {
        let original_payload = b"Test data 123";
        let original_packet = Packet::new(0xABCD, 0xEF01, 999, original_payload);

        // Serialize to bytes
        let bytes = original_packet.to_bytes();

        // Deserialize back to packet
        let deserialized_packet = Packet::from_bytes(&bytes);

        // Verify all fields match
        defmt::assert!(deserialized_packet.header.sender_id == original_packet.header.sender_id);
        defmt::assert!(deserialized_packet.header.target_id == original_packet.header.target_id);
        defmt::assert!(deserialized_packet.header.sequence_number == original_packet.header.sequence_number);
        defmt::assert!(deserialized_packet.header.payload_len == original_packet.header.payload_len);
        defmt::assert!(deserialized_packet.payload_data() == original_packet.payload_data());
        defmt::assert!(deserialized_packet == original_packet);
    }

    #[test]
    fn test_packet_serialization_with_control_flags() {
        let payload = b"Emergency!";
        let mut packet = Packet::new(0x1000, 0x2000, 555, payload);

        // Set some control flags
        packet.header.control.set_emergency(true);
        packet.header.control.set_ack_request(true);

        // Serialize and deserialize
        let bytes = packet.to_bytes();
        let deserialized = Packet::from_bytes(&bytes);

        // Verify control flags are preserved
        defmt::assert!(deserialized.header.control.is_emergency());
        defmt::assert!(deserialized.header.control.is_ack_request());
        defmt::assert!(!deserialized.header.control.is_ack());
        defmt::assert!(!deserialized.header.control.is_retransmit());
    }

    #[test]
    fn test_packet_size_constants() {
        // Verify that our constants are correct
        let packet = Packet::new(0, 0, 0, &[]);
        let bytes = packet.to_bytes();

        defmt::assert!(bytes.len() == PACKET_SIZE_BYTES);
        defmt::assert!(PACKET_SIZE_BYTES == core::mem::size_of::<Header>() + MAX_PAYLOAD_SIZE);
    }

    #[test]
    fn test_payload_data_boundary_conditions() {
        // Test with exactly MAX_PAYLOAD_SIZE
        let max_payload = [0x55; MAX_PAYLOAD_SIZE];
        let packet = Packet::new(0x1234, 0x5678, 1, &max_payload);

        defmt::assert!(packet.header.payload_len == MAX_PAYLOAD_SIZE as u8);
        defmt::assert!(packet.payload_data() == &max_payload[..]);

        // Test with one byte less than max
        let almost_max_payload = [0x66; MAX_PAYLOAD_SIZE - 1];
        let packet2 = Packet::new(0x1234, 0x5678, 2, &almost_max_payload);

        defmt::assert!(packet2.header.payload_len == (MAX_PAYLOAD_SIZE - 1) as u8);
        defmt::assert!(packet2.payload_data() == &almost_max_payload[..]);
    }

    #[test]
    fn test_packet_equality() {
        let payload = b"Same data";
        let packet1 = Packet::new(0x1111, 0x2222, 42, payload);
        let packet2 = Packet::new(0x1111, 0x2222, 42, payload);
        let packet3 = Packet::new(0x1111, 0x2222, 43, payload); // Different sequence

        defmt::assert!(packet1 == packet2);
        defmt::assert!(packet1 != packet3);
    }

    #[test]
    fn test_header_fields() {
        let packet = Packet::new(0xDEAD, 0xBEEF, 0xCAFE, b"test");

        defmt::assert!(packet.header.sender_id == 0xDEAD);
        defmt::assert!(packet.header.target_id == 0xBEEF);
        defmt::assert!(packet.header.sequence_number == 0xCAFE);
        defmt::assert!(packet.header.payload_len == 4);
    }
}

