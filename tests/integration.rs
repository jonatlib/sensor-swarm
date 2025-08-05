#![no_std]
#![no_main]

use defmt::info;
use defmt_semihosting as _;
// Replaced by semihosing, for testing, if logging works, remove this.
// use defmt_rtt as _; // global logger
use panic_probe as _;

// Custom defmt panic handler for tests
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

// Import the library modules we want to test

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use super::*;
    use sensor_swarm::radio::protocol::*;

    #[test]
    fn test_packet_creation_integration() {
        // Test actual packet creation functionality
        let packet = Packet::new(0x1234, 0x5678, 42, b"test");
        defmt::assert!(packet.header.sender_id == 0x1234);
        defmt::assert!(packet.header.target_id == 0x5678);
        defmt::assert!(packet.header.sequence_number == 42);
        defmt::assert!(packet.payload_data() == b"test");
    }
}
