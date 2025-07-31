#![no_std]
#![no_main]

use defmt_rtt as _; // global logger

// Custom defmt panic handler for tests
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

// Import the library modules we want to test

#[defmt_test::tests]
mod tests {
    use super::*;
    use sensor_swarm::radio::protocol::*;

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
        defmt::assert!(
            deserialized_packet.header.sequence_number == original_packet.header.sequence_number
        );
        defmt::assert!(
            deserialized_packet.header.payload_len == original_packet.header.payload_len
        );
        defmt::assert!(deserialized_packet.payload_data() == original_packet.payload_data());
    }
}
