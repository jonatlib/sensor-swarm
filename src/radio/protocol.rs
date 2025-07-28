// Radio protocol definitions
// This module defines the data structures for our custom radio packet format

use defmt::Format;

/// Maximum size of the packet payload in bytes
pub const MAX_PAYLOAD_SIZE: usize = 32;

/// Total packet size in bytes (header + payload)
pub const PACKET_SIZE_BYTES: usize = core::mem::size_of::<Header>() + MAX_PAYLOAD_SIZE;

/// Packet header containing routing and control information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
#[repr(transparent)]
pub struct PacketControl(pub u8);

impl PacketControl {
    /// Acknowledgment request flag bit position
    const ACK_REQUEST_BIT: u8 = 0;
    /// Acknowledgment response flag bit position
    const ACK_RESPONSE_BIT: u8 = 1;
    /// Emergency/priority flag bit position
    const EMERGENCY_BIT: u8 = 2;
    /// Retransmission flag bit position
    const RETRANSMIT_BIT: u8 = 3;

    /// Create a new PacketControl with default values
    pub const fn new() -> Self {
        Self(0)
    }

    /// Set the acknowledgment request flag
    pub fn set_ack_request(&mut self, value: bool) {
        if value {
            self.0 |= 1 << Self::ACK_REQUEST_BIT;
        } else {
            self.0 &= !(1 << Self::ACK_REQUEST_BIT);
        }
    }

    /// Check if acknowledgment is requested
    pub fn is_ack_request(&self) -> bool {
        (self.0 & (1 << Self::ACK_REQUEST_BIT)) != 0
    }

    /// Set the acknowledgment response flag
    pub fn set_ack_response(&mut self, value: bool) {
        if value {
            self.0 |= 1 << Self::ACK_RESPONSE_BIT;
        } else {
            self.0 &= !(1 << Self::ACK_RESPONSE_BIT);
        }
    }

    /// Check if this is an acknowledgment response
    pub fn is_ack(&self) -> bool {
        (self.0 & (1 << Self::ACK_RESPONSE_BIT)) != 0
    }

    /// Set the emergency/priority flag
    pub fn set_emergency(&mut self, value: bool) {
        if value {
            self.0 |= 1 << Self::EMERGENCY_BIT;
        } else {
            self.0 &= !(1 << Self::EMERGENCY_BIT);
        }
    }

    /// Check if this is an emergency packet
    pub fn is_emergency(&self) -> bool {
        (self.0 & (1 << Self::EMERGENCY_BIT)) != 0
    }

    /// Set the retransmission flag
    pub fn set_retransmit(&mut self, value: bool) {
        if value {
            self.0 |= 1 << Self::RETRANSMIT_BIT;
        } else {
            self.0 &= !(1 << Self::RETRANSMIT_BIT);
        }
    }

    /// Check if this is a retransmitted packet
    pub fn is_retransmit(&self) -> bool {
        (self.0 & (1 << Self::RETRANSMIT_BIT)) != 0
    }
}

impl Default for PacketControl {
    fn default() -> Self {
        Self::new()
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
        let header = unsafe {
            core::ptr::read_unaligned(bytes.as_ptr() as *const Header)
        };
        
        let mut payload = [0u8; MAX_PAYLOAD_SIZE];
        payload.copy_from_slice(&bytes[header_size..]);
        
        Self { header, payload }
    }
}