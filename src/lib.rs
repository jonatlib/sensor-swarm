#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use panic_probe as _;

// Module declarations
#[cfg(feature = "embedded")]
pub mod app;
#[cfg(feature = "embedded")]
pub mod hw;
pub mod radio;
#[cfg(feature = "embedded")]
pub mod sensors;
#[cfg(feature = "embedded")]
pub mod usb_commands;
