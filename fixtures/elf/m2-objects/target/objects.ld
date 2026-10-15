MEMORY
{
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 64K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}

ENTRY(Reset_Handler)

SECTIONS
{
  .text :
  {
    KEEP(*(.text.Reset_Handler))
    . = ALIGN(64);
    *(.text*)
    *(.rodata*)
    . = ALIGN(16);
  } > FLASH

  .data :
  {
    *(.data*)
  } > RAM AT > FLASH

  .bss (NOLOAD) :
  {
    *(.bss*)
    *(COMMON)
  } > RAM
}
