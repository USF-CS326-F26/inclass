/*
 * Linker script for the week 7 programs: the same layout as oslings'
 * asmlab.ld, plus a stack.
 *
 * QEMU's `virt` machine, started with `-bios none`, copies the ELF into RAM.
 * At reset its boot ROM at 0x1000 runs six instructions and jumps to the
 * RAM base, 0x8000_0000, where this script puts the ENTRY symbol.
 */

OUTPUT_ARCH( "riscv" )
ENTRY( _entry )            /* where the boot ROM's jump lands */

SECTIONS
{
  . = 0x80000000;         /* RAM base on the `virt` machine */

  .text : {
    *(.entry)             /* _entry must come first, at 0x80000000 */
    *(.text .text.*)
    . = ALIGN(0x1000);
    PROVIDE(etext = .);
  }

  .rodata : {
    . = ALIGN(16);
    *(.srodata .srodata.*)
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(16);
    *(.sdata .sdata.*)
    *(.data .data.*)
  }

  .bss : {
    . = ALIGN(16);
    *(.sbss .sbss.*)
    *(.bss .bss.*)
  }

  PROVIDE(end = .);

  /* 16 KiB of stack above everything else. Nothing is stored here in the
   * ELF; the stack grows down from __stack_top toward `end`. */
  . = ALIGN(16);
  . += 0x4000;
  PROVIDE(__stack_top = .);
}
