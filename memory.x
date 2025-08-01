/*
memory.x
Linker script for the STM32F401CE (512K Flash, 96K RAM)
*/

MEMORY
{
  /*
    The main program memory region. It MUST be named `FLASH` so that
    the default linker scripts can find it. Its size is reduced
    to make space for our virtual EEPROM.
  */
  FLASH          : ORIGIN = 0x08000000, LENGTH = 384K

  /*
    This region is reserved for the virtual EEPROM. No sections are placed
    here by default, so it remains available for the application to use.
  */
  EEPROM_VIRTUAL : ORIGIN = 0x08060000, LENGTH = 128K

  /* Main RAM for stack and variables. STM32F401 has 96K. */
  RAM            : ORIGIN = 0x20000000, LENGTH = 96K
}

/*
  Define symbols for the Rust code to access the EEPROM region.
  These symbols provide the start and end addresses.
*/
_eeprom_start = ORIGIN(EEPROM_VIRTUAL);
_eeprom_end = ORIGIN(EEPROM_VIRTUAL) + LENGTH(EEPROM_VIRTUAL);

/*
  DO NOT define a `SECTIONS` block here.
  The `cortex-m-rt` or `embassy-executor` crates provide a default
  linker script that handles the placement of .vector_table, .text,
  .data, and other sections into the `FLASH` and `RAM` regions
  defined above. Redefining `SECTIONS` here will cause overlap errors.
*/
