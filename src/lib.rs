#![no_std]
#![no_main]

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

#[cfg(test)]
#[defmt_test::tests]
mod tests {
    use defmt::assert;

    #[test]
    fn dummy_test() {
        assert!(true);
    }
}

#[cfg(test)]
use defmt_semihosting as _;
#[cfg(test)]
use panic_probe as _;

#[cfg(test)]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
