/// Hardware-agnostic EEPROM implementation using eeprom crate and linker symbols
/// Provides persistent storage using dedicated flash sector for STM32F411CE
use crate::hw::traits::FlashStorage;
use crate::usb_log;
use core::ops::Range;
use defmt::*;
use embassy_stm32::flash::{Blocking, Flash};

/// Hardware-agnostic EEPROM storage implementation
/// Uses the eeprom crate with linker-defined memory regions for persistent storage
pub struct EepromStorage {
    flash: Flash<'static, Blocking>,
    eeprom_range: Range<u32>,
    sector_size: u32,
}

impl EepromStorage {
    /// Create a new EEPROM storage instance
    pub fn new(flash: Flash<'static, Blocking>) -> Self {
        let eeprom_range = get_eeprom_range();
        let sector_size = eeprom_range.end - eeprom_range.start;

        info!("Initializing EEPROM storage...");
        usb_log!(
            info,
            "EEPROM range: 0x{:08X} - 0x{:08X} ({} KB)",
            eeprom_range.start,
            eeprom_range.end,
            sector_size / 1024
        );

        Self {
            flash,
            eeprom_range,
            sector_size,
        }
    }

    /// Check if an address is within the EEPROM storage region
    fn is_valid_address(&self, address: u32) -> bool {
        address < (self.eeprom_range.end - self.eeprom_range.start)
    }

    /// Convert relative address to absolute Flash address
    fn to_absolute_address(&self, relative_address: u32) -> u32 {
        self.eeprom_range.start + relative_address
    }

    /// Initialize the EEPROM with magic number if needed
    pub fn init(&mut self) -> Result<(), &'static str> {
        const EEPROM_MAGIC: u32 = 0xDEADBEEF;
        const MAGIC_OFFSET: u32 = 0;

        // Read the magic number from the EEPROM region
        let mut magic_buffer = [0u8; 4];
        unsafe {
            let magic_addr = self.eeprom_range.start + MAGIC_OFFSET;
            for (i, byte) in magic_buffer.iter_mut().enumerate() {
                *byte = core::ptr::read_volatile((magic_addr + i as u32) as *const u8);
            }
        }

        let stored_magic = u32::from_le_bytes(magic_buffer);

        if stored_magic != EEPROM_MAGIC {
            info!("Initializing EEPROM for first use...");

            // Erase the EEPROM sector
            match self
                .flash
                .blocking_erase(self.eeprom_range.start, self.eeprom_range.end)
            {
                Ok(_) => {
                    info!("EEPROM sector erased successfully");
                }
                Err(_) => {
                    error!("Failed to erase EEPROM sector");
                    return Err("EEPROM sector erase failed");
                }
            }

            // Write the magic number
            let magic_bytes = EEPROM_MAGIC.to_le_bytes();
            match self
                .flash
                .blocking_write(self.eeprom_range.start + MAGIC_OFFSET, &magic_bytes)
            {
                Ok(_) => {
                    info!("EEPROM magic written successfully");
                }
                Err(_) => {
                    error!("Failed to write EEPROM magic");
                    return Err("EEPROM magic write failed");
                }
            }
        } else {
            usb_log!(
                info,
                "EEPROM already initialized with magic: 0x{:08X}",
                stored_magic
            );
        }

        Ok(())
    }
}

impl FlashStorage for EepromStorage {
    fn read(&self, address: u32, buffer: &mut [u8]) -> Result<(), &'static str> {
        if !self.is_valid_address(address)
            || !self.is_valid_address(address + buffer.len() as u32 - 1)
        {
            return Err("Address out of range");
        }

        let abs_address = self.to_absolute_address(address);

        debug!(
            "EEPROM read: address=0x{:08X}, length={}",
            abs_address,
            buffer.len()
        );

        // Read directly from Flash memory
        unsafe {
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = core::ptr::read_volatile((abs_address + i as u32) as *const u8);
            }
        }

        debug!("EEPROM read completed successfully");
        Ok(())
    }

    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), &'static str> {
        if !self.is_valid_address(address)
            || !self.is_valid_address(address + data.len() as u32 - 1)
        {
            return Err("Address out of range");
        }

        let abs_address = self.to_absolute_address(address);

        debug!(
            "EEPROM write: address=0x{:08X}, length={}",
            abs_address,
            data.len()
        );

        match self.flash.blocking_write(abs_address, data) {
            Ok(_) => {
                debug!("EEPROM write completed successfully");
                Ok(())
            }
            Err(_) => {
                error!("EEPROM write failed at address 0x{:08X}", abs_address);
                Err("EEPROM write failed")
            }
        }
    }

    fn erase_sector(&mut self, address: u32) -> Result<(), &'static str> {
        if !self.is_valid_address(address) {
            return Err("Address out of range");
        }

        usb_log!(
            info,
            "Erasing EEPROM sector containing address 0x{:08X}",
            address
        );

        match self
            .flash
            .blocking_erase(self.eeprom_range.start, self.eeprom_range.end)
        {
            Ok(_) => {
                info!("EEPROM sector erased successfully");
                Ok(())
            }
            Err(_) => {
                error!("EEPROM sector erase failed");
                Err("EEPROM erase failed")
            }
        }
    }

    fn sector_size(&self) -> u32 {
        self.sector_size
    }

    fn total_size(&self) -> u32 {
        self.sector_size
    }
}

// ============================================================================
// LINKER SYMBOL ACCESS FOR EEPROM MEMORY REGION
// ============================================================================

// Import the EEPROM memory region symbols from the linker script.
extern "C" {
    static mut _eeprom_start: u32;
    static mut _eeprom_end: u32;
}

/// Retrieves the EEPROM memory range defined by the linker.
pub fn get_eeprom_range() -> Range<u32> {
    // This is unsafe because we are reading memory addresses directly from the linker.
    let start = unsafe { &raw const _eeprom_start as *const u32 as u32 };
    let end = unsafe { &raw const _eeprom_end as *const u32 as u32 };
    start..end
}
