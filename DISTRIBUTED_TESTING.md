# Distributed Testing with defmt-test

## Overview

This document explains how to organize tests across multiple modules while using `defmt-test` in an embedded no-std environment. The key insight is that `defmt-test` can work with distributed tests by creating separate test binaries for each module, avoiding symbol conflicts.

## The Problem

Previously, having multiple `#[defmt_test::tests]` attributes in different modules caused symbol conflicts:

```
error: symbol `DEFMT_TEST_COUNT` is already defined
```

This happened because each `#[defmt_test::tests]` attribute generates the same global symbols.

## The Solution: Separate Test Binaries

Instead of having tests directly in source modules, create separate test files in the `tests/` directory. Each test file becomes its own test binary with its own `#[defmt_test::tests]` attribute.

### Project Structure

```
src/
├── lib.rs                    # Main tests (consolidated)
├── commands/
│   └── parser.rs            # Source code (no tests)
└── ...
tests/
├── integration.rs           # Integration tests
├── parser.rs               # Parser-specific tests
└── ...                     # Additional test modules
```

## Implementation Steps

### 1. Create a Test File

Create a new test file in the `tests/` directory (e.g., `tests/parser.rs`):

```rust
#![no_std]
#![no_main]

use defmt_semihosting as _;
use panic_probe as _;

// Custom defmt panic handler for tests
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use super::*;
    use sensor_swarm::commands::parser::*;

    #[test]
    fn test_parse_sensors_command() {
        let parser = CommandParser::new();
        
        let result = parser.parse("sensors");
        defmt::assert!(result == Command::ReadSensors);
    }

    // Add more tests...
}
```

### 2. Configure in Cargo.toml

Add the test file configuration to `Cargo.toml`:

```toml
[[test]]
name = "parser"        # Must match the filename (tests/parser.rs)
harness = false        # Required for defmt-test
```

### 3. Test Structure Requirements

Each test file must include:

- `#![no_std]` and `#![no_main]` attributes
- `defmt_semihosting` and `panic_probe` imports
- Custom `#[defmt::panic_handler]`
- `#[cfg(test)]` and `#[defmt_test::tests]` attributes
- Import the modules you want to test

## Current Test Organization

### lib.rs Tests (10 tests)
- Radio protocol tests
- Hardware abstraction tests  
- Boot task tests
- Type conversion tests

### integration.rs Tests (1 test)
- Basic integration test

### parser.rs Tests (12 tests)
- Command parsing for all command types
- Case sensitivity testing
- Edge cases (empty commands, unknown commands)
- Partial match validation

## Running Tests

All tests can be run with a single command:

```bash
cargo test --features defmt-test,embedded
```

This will execute all test binaries:
- `Running unittests src/lib.rs` (10 tests)
- `Running tests/integration.rs` (1 test)  
- `Running tests/parser.rs` (12 tests)

**Total: 23 tests across 3 separate test binaries**

## Benefits of This Approach

1. **No Symbol Conflicts**: Each test binary has its own symbol space
2. **Modular Organization**: Tests can be organized by functionality
3. **Parallel Execution**: Test binaries can run independently
4. **Clear Separation**: Test code is separate from source code
5. **Scalable**: Easy to add new test modules

## Adding New Test Modules

To add tests for a new module (e.g., `src/sensors/temperature.rs`):

1. Create `tests/temperature.rs` with the standard test file structure
2. Add the configuration to `Cargo.toml`:
   ```toml
   [[test]]
   name = "temperature"
   harness = false
   ```
3. Import and test the temperature module functionality

## Best Practices

1. **Keep tests focused**: Each test file should focus on one module or related functionality
2. **Use descriptive names**: Test file names should clearly indicate what they test
3. **Comprehensive coverage**: Test normal cases, edge cases, and error conditions
4. **Hardware-agnostic**: Keep tests runnable in QEMU without real hardware
5. **Use defmt assertions**: Always use `defmt::assert!` instead of standard `assert!`

## Example Test Coverage

The parser tests demonstrate comprehensive coverage:

- ✅ All command types (sensors, temperature, debug, etc.)
- ✅ Case insensitivity 
- ✅ Multiple command aliases (temp/temperature, help/?)
- ✅ Unknown command handling
- ✅ Empty input handling
- ✅ Partial match rejection
- ✅ Default trait implementation

This approach successfully resolves the distributed testing challenge while maintaining clean code organization and comprehensive test coverage.