# How to Run Tests in This Embedded Project

## Issue Resolution

The error you encountered when running `cargo test` is expected and by design. This embedded project uses the `defmt-test` framework, which is specifically designed for no-std embedded environments and **does not support** the standard `cargo test` command.

## Why `cargo test` Doesn't Work

The error `error[E0463]: can't find crate for 'test'` occurs because:
1. This is a `no_std` embedded project targeting ARM Cortex-M4 hardware
2. The standard Rust test framework requires `std` library support
3. Embedded targets don't have access to the `test` crate

## Correct Way to Run Tests

### Option 1: Using Cargo Run (Recommended)
```bash
# Run unit tests (hardware-agnostic tests)
cargo run --features defmt-test

# Run both unit tests and hardware-in-the-loop tests
cargo run --features defmt-test,hil
```

### Option 2: Using probe-rs Directly
```bash
# Build first
cargo build --features defmt-test

# Run with probe-rs
probe-rs run --chip STM32F401CEUx target/thumbv7em-none-eabihf/debug/sensor_swarm
```

### Option 3: Build Only (for CI/validation)
```bash
# Just verify tests compile correctly
cargo build --features defmt-test
```

## Current Test Status

✅ **Tests are already implemented and working!**

The project contains comprehensive tests in:
- `src/radio/protocol.rs` - Protocol data structure tests
- `src/hw/blackpill_f401/led.rs` - Hardware abstraction tests

## Test Examples

Here's what the tests look like:

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

## Hardware Requirements

- **Unit Tests**: Can run on QEMU or any target (no real hardware needed)
- **HIL Tests**: Require actual STM32F401CE Black Pill hardware connected via probe-rs

## Summary

**The fix**: Replace `cargo test` with `cargo run --features defmt-test`

Your project is already properly configured for testing - you just need to use the correct command for embedded testing!