/*
 * Controlled layout for the P0 memory-accounting proof.
 *
 * RAM  (rwx) is writable, therefore live storage.
 * ROM  (rx)  is not writable, therefore the load image.
 *
 * `.data` and `.ota` are VMA-resident in RAM but LMA-stored in ROM (AT> ROM), which is the
 * exact shape that makes nonvolatile footprint and runtime RAM footprint two different
 * numbers. A `.data`-by-name rule would produce the same totals here by luck; the proof is
 * that PT_LOAD carries both addresses independently.
 */
MEMORY {
  ROM (rx)  : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS {
  .text : { *(.text*) } > ROM

  .rodata : { *(.rodata*) } > ROM

  _sidata = LOADADDR(.data);
  .data : { *(.data*) } > RAM AT> ROM

  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM

  .ota : { *(.ota_staging*) } > RAM AT> ROM
}
