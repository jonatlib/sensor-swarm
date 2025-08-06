// Radio protocol definitions
// This module defines the data structures for our custom radio packet format

use bitfield_struct::bitfield;
use defmt::Format;

/// Maximum size of the packet payload in bytes
pub const MAX_PAYLOAD_SIZE: usize = 32;

/// Total packet size in bytes (header + payload)
pub const PACKET_SIZE_BYTES: usize = core::mem::size_of::<Header>() + MAX_PAYLOAD_SIZE;

/// Packet header containing routing and control information
#[derive(Debug, Clone, PartialEq, Eq, Format)]
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
#[derive(PartialEq, Eq, Format)]
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

/// Complete radio packet structure
#[derive(Debug, Clone, PartialEq, Eq, Format)]
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
    /// TODO: Add Reed-Solomon error correction encoding as per project requirements
    /// TODO: Add packet integrity checks (CRC/checksum) for production reliability
    pub fn to_bytes(&self) -> [u8; PACKET_SIZE_BYTES] {
        let mut bytes = [0u8; PACKET_SIZE_BYTES];

        // TODO: Replace unsafe pointer operations with safe serialization
        // This unsafe code should be replaced with safer alternatives for production
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
    /// TODO: Add Reed-Solomon error correction decoding as per project requirements
    /// TODO: Add packet validation and integrity checks for production reliability
    /// TODO: Add error handling for malformed or corrupted packets
    pub fn from_bytes(bytes: &[u8; PACKET_SIZE_BYTES]) -> Self {
        let header_size = core::mem::size_of::<Header>();

        // TODO: Replace unsafe unaligned read with safe deserialization
        // This unsafe code should be replaced with safer alternatives for production
        // Deserialize header
        let header = unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const Header) };

        let mut payload = [0u8; MAX_PAYLOAD_SIZE];
        payload.copy_from_slice(&bytes[header_size..]);

        Self { header, payload }
    }
}
