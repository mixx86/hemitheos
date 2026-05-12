ENTRY(_start)

SECTIONS
{
    . = 0x40080000;

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
        __bss_end = .;
    }

    . = ALIGN(16);
    __hypervisor_end = .;

    . = ALIGN(16);
    . += 0x10000; /* 64 KiB stack */
    stack_top = .;
}
