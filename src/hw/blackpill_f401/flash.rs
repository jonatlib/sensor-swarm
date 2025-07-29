/// Flash/EEPROM implementation for STM32F401 Black Pill
/// Provides hardware-specific persistent storage using Flash memory as EEPROM

use crate::hw::traits::FlashStorage;
use embassy_stm32::flash::{Flash, Blocking};
use defmt::*;

/// Flash-based EEPROM implementation for STM32F401 Black Pill
/// Uses the last sector of Flash memory for persistent storage
pub struct BlackPillFlashStorage {
    flash: Flash<'static, Blocking>,
    storage_start_address: u32,
    storage_size: u32,
    sector_size: u32,
}

impl BlackPillFlashStorage {
    /// Flash memory layout for STM32F401CCU6 (256KB Flash)
    /// Sector 0: 0x08000000 - 0x08003FFF (16KB)
    /// Sector 1: 0x08004000 - 0x08007FFF (16KB)
    /// Sector 2: 0x08008000 - 0x0800BFFF (16KB)
    /// Sector 3: 0x0800C000 - 0x0800FFFF (16KB)
    /// Sector 4: 0x08010000 - 0x0801FFFF (64KB)
    /// Sector 5: 0x08020000 - 0x0803FFFF (128KB) - Used for EEPROM storage
    const FLASH_BASE: u32 = 0x08000000;
    const TOTAL_FLASH_SIZE: u32 = 256 * 1024; // 256KB
    const EEPROM_SECTOR_START: u32 = 0x08020000; // Sector 5 start
    const EEPROM_SECTOR_SIZE: u32 = 128 * 1024; // 128KB
    const EEPROM_STORAGE_SIZE: u32 = 64 * 1024; // Use 64KB for storage, leave rest as safety margin

    /// Create a new Flash storage instance
    pub fn new(flash: Flash<'static, Blocking>) -> Self {
        info!("Initializing Flash storage...");
        info!("EEPROM region: 0x{:08X} - 0x{:08X} ({} KB)", 
              Self::EEPROM_SECTOR_START, 
              Self::EEPROM_SECTOR_START + Self::EEPROM_STORAGE_SIZE,
              Self::EEPROM_STORAGE_SIZE / 1024);
        
        Self {
            flash,
            storage_start_address: Self::EEPROM_SECTOR_START,
            storage_size: Self::EEPROM_STORAGE_SIZE,
            sector_size: Self::EEPROM_SECTOR_SIZE,
        }
    }

    /// Check if an address is within the storage region
    fn is_valid_address(&self, address: u32) -> bool {
        address < self.storage_size
    }

    /// Convert relative address to absolute Flash address
    fn to_absolute_address(&self, relative_address: u32) -> u32 {
        self.storage_start_address + relative_address
    }

    /// Get storage region information
    pub fn get_storage_info(&self) -> FlashStorageInfo {
        FlashStorageInfo {
            start_address: self.storage_start_address,
            size: self.storage_size,
            sector_size: self.sector_size,
            page_size: 4, // STM32F401 has 4-byte minimum write size
            erase_value: 0xFF,
        }
    }

    /// Check if a region is erased (all bytes are 0xFF)
    pub fn is_erased(&self, address: u32, length: u32) -> Result<bool, &'static str> {
        if !self.is_valid_address(address) || !self.is_valid_address(address + length - 1) {
            return Err("Address out of range");
        }

        let abs_address = self.to_absolute_address(address);
        
        // Read the region and check if all bytes are 0xFF
        for offset in 0..length {
            let byte_addr = abs_address + offset;
            let byte_value = unsafe { core::ptr::read_volatile(byte_addr as *const u8) };
            if byte_value != 0xFF {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// Format the entire storage region (erase all sectors)
    pub fn format(&mut self) -> Result<(), &'static str> {
        info!("Formatting Flash storage region...");
        
        match self.flash.blocking_erase(self.storage_start_address, self.storage_start_address + self.sector_size) {
            Ok(_) => {
                info!("Flash storage formatted successfully");
                Ok(())
            }
            Err(_) => {
                error!("Failed to format Flash storage");
                Err("Flash format failed")
            }
        }
    }
}

impl FlashStorage for BlackPillFlashStorage {
    fn read(&self, address: u32, buffer: &mut [u8]) -> Result<(), &'static str> {
        if !self.is_valid_address(address) || !self.is_valid_address(address + buffer.len() as u32 - 1) {
            return Err("Address out of range");
        }

        let abs_address = self.to_absolute_address(address);
        
        debug!("Flash read: address=0x{:08X}, length={}", abs_address, buffer.len());
        
        // Read directly from Flash memory
        unsafe {
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = core::ptr::read_volatile((abs_address + i as u32) as *const u8);
            }
        }
        
        debug!("Flash read completed successfully");
        Ok(())
    }
    
    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), &'static str> {
        if !self.is_valid_address(address) || !self.is_valid_address(address + data.len() as u32 - 1) {
            return Err("Address out of range");
        }

        let abs_address = self.to_absolute_address(address);
        
        debug!("Flash write: address=0x{:08X}, length={}", abs_address, data.len());
        
        match self.flash.blocking_write(abs_address, data) {
            Ok(_) => {
                debug!("Flash write completed successfully");
                Ok(())
            }
            Err(_) => {
                error!("Flash write failed at address 0x{:08X}", abs_address);
                Err("Flash write failed")
            }
        }
    }
    
    fn erase_sector(&mut self, address: u32) -> Result<(), &'static str> {
        if !self.is_valid_address(address) {
            return Err("Address out of range");
        }

        // For STM32F401, we erase the entire sector containing the address
        let abs_address = self.to_absolute_address(address);
        
        info!("Erasing Flash sector containing address 0x{:08X}", abs_address);
        
        match self.flash.blocking_erase(self.storage_start_address, self.storage_start_address + self.sector_size) {
            Ok(_) => {
                info!("Flash sector erased successfully");
                Ok(())
            }
            Err(_) => {
                error!("Flash sector erase failed");
                Err("Flash erase failed")
            }
        }
    }
    
    fn sector_size(&self) -> u32 {
        self.sector_size
    }
    
    fn total_size(&self) -> u32 {
        self.storage_size
    }
}

/// Flash storage manager for organizing Flash operations
pub struct BlackPillFlashManager {
    initialized: bool,
}

impl BlackPillFlashManager {
    /// Create a new Flash manager
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize the Flash manager
    pub fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing Flash manager...");
        
        // Flash initialization is handled per-instance basis
        // This method can be used for any global Flash setup if needed
        
        self.initialized = true;
        info!("Flash manager initialized successfully");
        Ok(())
    }

    /// Check if Flash manager has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get information about Flash memory layout
    pub fn get_flash_info(&self) -> FlashInfo {
        FlashInfo {
            total_flash_size: BlackPillFlashStorage::TOTAL_FLASH_SIZE,
            flash_base_address: BlackPillFlashStorage::FLASH_BASE,
            eeprom_start_address: BlackPillFlashStorage::EEPROM_SECTOR_START,
            eeprom_size: BlackPillFlashStorage::EEPROM_STORAGE_SIZE,
            sector_layout: &[
                ("Sector 0", 0x08000000, 16 * 1024),
                ("Sector 1", 0x08004000, 16 * 1024),
                ("Sector 2", 0x08008000, 16 * 1024),
                ("Sector 3", 0x0800C000, 16 * 1024),
                ("Sector 4", 0x08010000, 64 * 1024),
                ("Sector 5 (EEPROM)", 0x08020000, 128 * 1024),
            ],
            write_alignment: 4, // 4-byte alignment required
            erase_value: 0xFF,
        }
    }
}

impl Default for BlackPillFlashManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Flash storage information structure
#[derive(Debug, Clone)]
pub struct FlashStorageInfo {
    pub start_address: u32,
    pub size: u32,
    pub sector_size: u32,
    pub page_size: u32,
    pub erase_value: u8,
}

/// Flash information structure
#[derive(Debug, Clone)]
pub struct FlashInfo {
    pub total_flash_size: u32,
    pub flash_base_address: u32,
    pub eeprom_start_address: u32,
    pub eeprom_size: u32,
    pub sector_layout: &'static [(&'static str, u32, u32)], // (name, address, size)
    pub write_alignment: u32,
    pub erase_value: u8,
}

/// Simple key-value store implementation using Flash storage
/// Provides a higher-level interface for storing configuration data
pub struct BlackPillKeyValueStore {
    storage: BlackPillFlashStorage,
    initialized: bool,
}

impl BlackPillKeyValueStore {
    /// Create a new key-value store
    pub fn new(storage: BlackPillFlashStorage) -> Self {
        Self {
            storage,
            initialized: false,
        }
    }

    /// Initialize the key-value store
    pub fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing key-value store...");
        
        // Check if the storage region is formatted
        match self.storage.is_erased(0, 1024) {
            Ok(true) => {
                info!("Storage region is clean, ready for use");
            }
            Ok(false) => {
                info!("Storage region contains data");
            }
            Err(e) => {
                error!("Failed to check storage region: {}", e);
                return Err(e);
            }
        }
        
        self.initialized = true;
        info!("Key-value store initialized successfully");
        Ok(())
    }

    /// Store a value with a key (simplified implementation)
    /// In a full implementation, this would include proper key management
    pub fn store(&mut self, key: &str, value: &[u8]) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Key-value store not initialized");
        }

        if key.len() > 32 || value.len() > 256 {
            return Err("Key or value too large");
        }

        info!("Storing key '{}' with {} bytes", key, value.len());
        
        // Simple implementation: store at fixed offset based on key hash
        let key_hash = key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)) % 1000;
        let address = key_hash * 300; // 300 bytes per entry (32 key + 4 length + 256 value + padding)
        
        // Store key length + key + value length + value
        let mut buffer = [0u8; 300];
        let key_bytes = key.as_bytes();
        
        buffer[0] = key_bytes.len() as u8;
        buffer[1..1 + key_bytes.len()].copy_from_slice(key_bytes);
        buffer[33] = value.len() as u8;
        buffer[34..34 + value.len()].copy_from_slice(value);
        
        self.storage.write(address, &buffer)
    }

    /// Retrieve a value by key (simplified implementation)
    pub fn retrieve(&self, key: &str, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if !self.initialized {
            return Err("Key-value store not initialized");
        }

        let key_hash = key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32)) % 1000;
        let address = key_hash * 300;
        
        let mut entry_buffer = [0u8; 300];
        self.storage.read(address, &mut entry_buffer)?;
        
        let stored_key_len = entry_buffer[0] as usize;
        if stored_key_len == 0 || stored_key_len > 32 {
            return Err("No data found for key");
        }
        
        let stored_key = &entry_buffer[1..1 + stored_key_len];
        if stored_key != key.as_bytes() {
            return Err("Key not found");
        }
        
        let value_len = entry_buffer[33] as usize;
        if value_len > buffer.len() {
            return Err("Buffer too small");
        }
        
        buffer[..value_len].copy_from_slice(&entry_buffer[34..34 + value_len]);
        Ok(value_len)
    }
}