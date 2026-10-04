/*
 * Case D, a custom writable region holding a buffer pool. The claim under test is narrow: an
 * unfamiliar region name must not make its memory disappear. Nothing in this file says anything
 * about caches, because a linker script cannot.
 */
MEMORY {
  FLASH  (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM    (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
  DMARAM (rw)  : ORIGIN = 0x30000000, LENGTH = 16K
}
SECTIONS {
  .text       : { *(.text*) } > FLASH
  .rodata     : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data       : { *(.data*) } > RAM AT> FLASH
  .dma_buffer : { *(.dma_buffer*) } > DMARAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
