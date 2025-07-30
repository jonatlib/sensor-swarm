# Testing Guide for Sensor Swarm Project

## Overview

This project uses the `defmt-test` framework for embedded testing in a no-std environment. The tests are embedded within the source modules and use a custom feature flag system to avoid conflicts with the standard Rust test framework.

## Test Types

### Unit Tests
- **Location**: Embedded within source modules (e.g., `src/radio/protocol.rs`)
- **Purpose**: Test pure logic, data structures, and algorithms
- **Hardware Requirements**: None - can run on QEMU or any target
- **Feature Gate**: `defmt-test`

### Hardware-in-the-Loop (HIL) Tests
- **Location**: Embedded within source modules
- **Purpose**: Test real hardware peripherals (GPIO, SPI, timers, etc.)
- **Hardware Requirements**: Real hardware or accurate hardware simulation
- **Feature Gate**: `hil` (in addition to `defmt-test`)

## Compilation

### Regular Library Compilation
```bash
# Compile the library without tests (default behavior)
cargo build --lib

# This will NOT include any test code and avoids the "can't find crate for `test`" error
```

### Test Compilation
```bash
# Compile the library with embedded unit tests
cargo build --lib --features defmt-test

# Compile with both unit tests and HIL tests
cargo build --lib --features defmt-test,hil
```

## Running Tests

### Prerequisites
- Hardware target (STM32F401CE Black Pill) or QEMU
- `probe-rs` installed for hardware testing
- Proper `.cargo/config.toml` configuration

### Unit Tests (QEMU or Hardware)
```bash
# Run unit tests on hardware
probe-rs run --chip STM32F401CEUx target/thumbv7em-none-eabihf/debug/sensor_swarm --features defmt-test

# Or use the configured runner
cargo run --features defmt-test
```

### HIL Tests (Hardware Only)
```bash
# Run HIL tests - requires real hardware
probe-rs run --chip STM32F401CEUx target/thumbv7em-none-eabihf/debug/sensor_swarm --features defmt-test,hil
```

## Test Structure

### Example Unit Test
```rust
#[cfg(feature = "defmt-test")]
#[defmt_test::tests]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let payload = b"Hello, World!";
        let packet = Packet::new(0x1234, 0x5678, 42, payload);
        
        defmt::assert!(packet.header.sender_id == 0x1234);
        defmt::assert!(packet.payload_data() == payload);
    }
}
```

### Example HIL Test
```rust
#[cfg(all(feature = "defmt-test", feature = "hil"))]
#[defmt_test::tests]
mod hardware_tests {
    use super::*;
    use embassy_time::{Duration, Timer};

    #[test]
    async fn test_gpio_functionality() {
        // Test code that requires real hardware
        let p = embassy_stm32::init(Default::default());
        let mut led = Output::new(p.PC13, Level::High, Speed::Low);
        
        led.set_low();
        Timer::after(Duration::from_millis(100)).await;
        
        // Verify hardware state
        defmt::assert!(true); // Replace with actual hardware verification
    }
}
```

## Key Differences from Standard Rust Testing

1. **No `#[cfg(test)]`**: We use `#[cfg(feature = "defmt-test")]` instead to avoid the standard test framework
2. **No `cargo test`**: Use `cargo run` with features or direct `probe-rs` commands
3. **`defmt::assert!`**: Use defmt assertions instead of standard `assert!`
4. **Embedded execution**: Tests run on the target hardware/emulator, not on the host

## Troubleshooting

### "can't find crate for `test`" Error
This error occurs when using `#[cfg(test)]` in a no-std environment. Our solution:
- Use `#[cfg(feature = "defmt-test")]` instead
- Make `defmt-test` an optional dependency
- Compile with explicit feature flags

### Tests Not Running
- Ensure you're using the `--features defmt-test` flag
- Check that your hardware is connected and `probe-rs` is configured
- Verify the chip name in `.cargo/config.toml` matches your hardware

### Missing Test Output
- Tests use `defmt` for output, which requires `defmt-rtt` and proper RTT configuration
- Ensure your debugger/probe supports RTT output