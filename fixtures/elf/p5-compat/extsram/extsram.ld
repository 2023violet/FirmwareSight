/*
 * Case C, external SRAM. EXTSRAM is a distinct declared region whose address is outside every
 * internal-RAM assumption, so the product can only know it from the MAP Memory Configuration table.
 */
MEMORY {
  FLASH   (rx)   : ORIGIN = 0x08000000, LENGTH = 256K
  RAM     (rwx)  : ORIGIN = 0x20000000, LENGTH = 64K
  EXTSRAM (rwx)  : ORIGIN = 0x60000000, LENGTH = 512K
}
SECTIONS {
  .text         : { *(.text*) } > FLASH
  .rodata       : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data         : { *(.data*) } > RAM AT> FLASH
  .extsram_pool : { *(.extsram_pool*) } > EXTSRAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
