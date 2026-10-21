ENTRY(start)

#include <asm/config.h>
#include <macros.h>


SECTIONS
{
    . = HEMITHEOS_VIRT_START;
    _start = .;

    .text.boot : ALIGN(16) {
        KEEP(*(.text.boot))
    }

    .exception : ALIGN(2048) {
        KEEP(*(.text.exception*))
    }

    .text : ALIGN(16) {
        *(.text .text.*)
    }

    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(16) {
        *(.data .data.*)
    }

    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4096);
        boot_pgtable = .;
        . += 0x1000;
        boot_first = .;
        . += 0x1000;
        boot_second = .;
        . += 0x1000;
        boot_third = .;
        . += 0x1000;
        boot_second_id = .;
        . += 0x1000;
        __bss_end = .;
    }

    . = ALIGN(16);
    __hypervisor_end = .;

    . = ALIGN(16);
    . += 0x10000;
    stack_top = .; 
    _end = .;
}

ASSERT((_end - _start) <= HEMITHEOS_VIRT_SIZE, "Hemitheos is too big")

