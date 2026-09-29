/*
 * Controlled ROM/RAM layout for the P2 Compare fixture pair.
 *
 * The two sides differ in exactly one rule each: this script declares only the custom section the
 * side it belongs to actually feeds. A rule with no matching input object would still emit an empty
 * output section, and an empty `.calib` in the target would be reported as a shrink rather than as
 * the removal it is.
 *
 * RAM (rwx) is writable, therefore live storage. ROM (rx) is not writable, therefore the load
 * image. `.ota` is VMA-resident in RAM and LMA-stored in ROM, so one section charges both budgets.
 */
MEMORY {
  ROM (rx)  : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS {
  .text : { *(.text*) } > ROM

  .rodata : { *(.rodata*) } > ROM

  /*
   * Pinned to a fixed flash address, and fed by identical content on both sides, so this is the
   * section a correct diff must report as unchanged: same size, same VMA, same LMA.
   */
  .settings 0x08010000 : { *(.settings*) } > ROM

  _sidata = LOADADDR(.data);
  .data : { *(.data*) } > RAM AT> ROM

  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM

  /* Base only: 32 calibration bytes that the target build no longer carries. */
  .calib : { *(.calib*) } > ROM
}
