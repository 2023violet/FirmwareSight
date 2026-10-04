/*
 * Case B, executable resident in RAM. `.fast_text` runs from RAM and is stored in FLASH, so VMA and
 * LMA disagree for a section holding code. A rule that equates `.text` with the nonvolatile side
 * charges this section wrongly on purpose-built evidence.
 */
MEMORY {
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
SECTIONS {
  .text      : { *(.text*) } > FLASH
  .rodata    : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data      : { *(.data*) } > RAM AT> FLASH
  .fast_text : { *(.fast_text*) } > RAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
