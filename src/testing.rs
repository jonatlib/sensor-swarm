/// Testing utilities and mock implementations
/// This module provides hardware-agnostic mock implementations for testing
/// without requiring actual hardware peripherals.

pub mod blackpill_f401;
#[cfg(feature = "hil")]
pub mod hil;