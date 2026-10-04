/*
 * Plain FLASH/RAM split, shared by the debug/no-debug pair, the Clang fixture and the long-preamble
 * fixture, so that the compiler or the debug presence is the only variable between them and a
 * baseline.
 */
MEMORY {
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
SECTIONS {
  .text   : { *(.text*) } > FLASH
  .rodata : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data   : { *(.data*) } > RAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
