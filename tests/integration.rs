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
    fn test_integration_basic() {
        // Basic integration test to verify the test framework is working
        defmt::assert!(true);
    }
}
