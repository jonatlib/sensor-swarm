/*
memory.x
Linker script for the STM32F411CEU6 (512K Flash, 128K RAM)
*/

MEMORY
{
  /*
    The STM32F411 flash is organized into sectors that must be erased entirely.
    To prevent data corruption, we reserve the last 128KB sector (Sector 7)
    exclusively for the virtual EEPROM.
  */
  FLASH_PROG     : ORIGIN = 0x08000000, LENGTH = 384K
  EEPROM_VIRTUAL : ORIGIN = 0x08060000, LENGTH = 128K
  RAM            : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Base address of the stack */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* Define symbols for the EEPROM region for Rust code to access */
_eeprom_start = ORIGIN(EEPROM_VIRTUAL);
_eeprom_end = ORIGIN(EEPROM_VIRTUAL) + LENGTH(EEPROM_VIRTUAL);

/* Instruct the linker to place sections correctly */
SECTIONS
{
  .text :
  {
    *(.vectors*)
    *(.text*)
    *(.rodata*)
    . = ALIGN(4);
  } > FLASH_PROG

  .data : { . = ALIGN(4); _sdata = .; *(.data*); . = ALIGN(4); _edata = .; } > RAM AT > FLASH_PROG
  .bss :  { . = ALIGN(4); _sbss = .; *(.bss*); *(COMMON); . = ALIGN(4); _ebss = .; } > RAM
}

