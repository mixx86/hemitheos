#include <linkage.h>
#include <asm/config.h>
#include <asm/page.h>
#include <asm/sysregs.h>

// Atomic flag used to elect the primary CPU at boot.
// Lives in .data so it is zero when the image is loaded, before BSS is cleared.
.section .data
DATA_LOCAL(_boot_cpu_online, 4)
    .word 0
END(_boot_cpu_online)

/*
 * __HEAD_FLAGS encodes the page granule (bits [2:1]) and the fixed physical
 * base requirement (bit 3) into the ARM64 Image header flags field.
 */
#define __HEAD_FLAG_PAGE_SIZE   ((PAGE_SHIFT - 10) / 2)
#define __HEAD_FLAG_PHYS_BASE   1
#define __HEAD_FLAGS            ((__HEAD_FLAG_PAGE_SIZE << 1) | \
                                 (__HEAD_FLAG_PHYS_BASE << 3))

.section .text.boot

/*
 * ARM64 Image header — Linux ARM64 boot protocol (Documentation/arm64/booting.rst).
 * The image is also a valid PE/COFF binary when HEMITHEOS_EFI is defined,
 * allowing UEFI firmware to load it directly without a shim.
 *
 * Offset  Field
 *  0x00   code0      first instruction — "MZ" opcode for PE/COFF detection
 *  0x04   code1      branch to _start
 *  0x08   text_offset
 *  0x10   image_size
 *  0x18   flags      __HEAD_FLAGS
 *  0x20   reserved × 3
 *  0x38   magic      "ARM\x64"
 *  0x3c   pe_offset  0 (no PE header) or offset to pe_header
 */
FUNC_ENTRY(start)
efi_head:
    /*
     * This add instruction has no meaningful effect at runtime.
     * Its encoding forms the "MZ" magic signature required for
     * PE/COFF detection by UEFI firmware.
     */
    add     x13, x18, #0x16
    b       real_start
END(start)
    .quad   0                               /* text_offset: RAM_BASE + 0      */
    .quad   _end - start                    /* effective image size                 */
    .quad   __HEAD_FLAGS                    /* page granule + fixed phys base       */
    .quad   0                               /* reserved                             */
    .quad   0                               /* reserved                             */
    .quad   0                               /* reserved                             */
    .byte   0x41                            /* magic "ARM\x64"                      */
    .byte   0x52
    .byte   0x4d
    .byte   0x64
#ifndef HEMITHEOS_EFI
    .long   0                               /* 0 = no PE/COFF header                */
#else
    .long   pe_header - efi_head            /* offset to PE/COFF header             */

    /*
     * PE/COFF header for UEFI boot.
     * The arm64 Image format is designed to be simultaneously a Linux
     * Image and a valid PE/COFF binary.
     * Spec: https://docs.microsoft.com/en-us/windows/win32/debug/pe-format
     */
    .align  3
pe_header:
    .ascii  "PE"
    .short  0
coff_header:
    .short  0xaa64                          /* Machine: AArch64                     */
    .short  2                               /* NumberOfSections                     */
    .long   0                               /* TimeDateStamp                        */
    .long   0                               /* PointerToSymbolTable                 */
    .long   1                               /* NumberOfSymbols                      */
    .short  section_table - optional_header /* SizeOfOptionalHeader                 */
    .short  0x206                           /* Characteristics:                     */
                                            /*   IMAGE_FILE_DEBUG_STRIPPED        | */
                                            /*   IMAGE_FILE_EXECUTABLE_IMAGE      | */
                                            /*   IMAGE_FILE_LINE_NUMS_STRIPPED      */
optional_header:
    .short  0x20b                           /* PE32+ format                         */
    .byte   0x02                            /* MajorLinkerVersion                   */
    .byte   0x14                            /* MinorLinkerVersion                   */
    .long   _end - real_start               /* SizeOfCode                           */
    .long   0                               /* SizeOfInitializedData                */
    .long   0                               /* SizeOfUninitializedData              */
    .long   efi_entry - efi_head            /* AddressOfEntryPoint                  */
    .long   real_start - efi_head               /* BaseOfCode                           */

extra_header_fields:
    .quad   0                               /* ImageBase                            */
    .long   0x1000                          /* SectionAlignment (4 KiB)             */
    .long   0x8                             /* FileAlignment                        */
    .short  0                               /* MajorOperatingSystemVersion          */
    .short  0                               /* MinorOperatingSystemVersion          */
    .short  0                               /* MajorImageVersion                    */
    .short  0                               /* MinorImageVersion                    */
    .short  0                               /* MajorSubsystemVersion                */
    .short  0                               /* MinorSubsystemVersion                */
    .long   0                               /* Win32VersionValue                    */
    .long   _end - efi_head                 /* SizeOfImage                          */
    .long   real_start - efi_head           /* SizeOfHeaders                        */
    .long   0                               /* CheckSum                             */
    .short  0xa                             /* Subsystem: EFI application           */
    .short  0                               /* DllCharacteristics                   */
    .quad   0                               /* SizeOfStackReserve                   */
    .quad   0                               /* SizeOfStackCommit                    */
    .quad   0                               /* SizeOfHeapReserve                    */
    .quad   0                               /* SizeOfHeapCommit                     */
    .long   0                               /* LoaderFlags                          */
    .long   0x6                             /* NumberOfRvaAndSizes                  */

    .quad   0                               /* ExportTable                          */
    .quad   0                               /* ImportTable                          */
    .quad   0                               /* ResourceTable                        */
    .quad   0                               /* ExceptionTable                       */
    .quad   0                               /* CertificationTable                   */
    .quad   0                               /* BaseRelocationTable                  */

section_table:
    /*
     * Dummy .reloc section — the EFI loader requires at least one relocation
     * section to declare the image as relocatable, even if it is empty.
     */
    .ascii  ".reloc"
    .byte   0
    .byte   0                               /* 8-byte name, zero-padded             */
    .long   0
    .long   0
    .long   0                               /* SizeOfRawData                        */
    .long   0                               /* PointerToRawData                     */
    .long   0                               /* PointerToRelocations                 */
    .long   0                               /* PointerToLineNumbers                 */
    .short  0                               /* NumberOfRelocations                  */
    .short  0                               /* NumberOfLineNumbers                  */
    .long   0x42100040                      /* Characteristics                      */

    .ascii  ".text"
    .byte   0
    .byte   0
    .byte   0                               /* 8-byte name, zero-padded             */
    .long   _end - real_start                   /* VirtualSize                          */
    .long   real_start - efi_head               /* VirtualAddress                       */
    .long   _end - real_start                   /* SizeOfRawData                        */
    .long   real_start - efi_head               /* PointerToRawData                     */
    .long   0                               /* PointerToRelocations                 */
    .long   0                               /* PointerToLineNumbers                 */
    .short  0                               /* NumberOfRelocations                  */
    .short  0                               /* NumberOfLineNumbers                  */
    .long   0xe0500020                      /* Characteristics                      */
    .align  5
#endif /* HEMITHEOS_EFI */

FUNC(real_start)
    msr     DAIFSet, #0xf                   /* Disable all interrupts */
END(real_start)

FUNC_LOCAL(elect_primary_cpu)
    // The first CPU to claim _boot_cpu_online proceeds; all others are parked.
    // Topology-agnostic: works regardless of MPIDR values, big.LITTLE, or cluster layout.
    // On PSCI systems secondaries never reach here anyway.
    adrp    x1, _boot_cpu_online
    add     x1, x1, :lo12:_boot_cpu_online
    mov     w2, #1
.Lclaim:
    ldaxr   w3, [x1]
    cbnz    w3, init_secondary              // already claimed — we are secondary
    stlxr   w4, w2, [x1]                   // w4=0 on success
    cbnz    w4, .Lclaim                    // lost exclusive monitor — retry
    // Only the primary CPU reaches here.
    // x20 is callee-saved so it survives the bl to main.
    mov     x20, x0
END(elect_primary_cpu)

FUNC_LOCAL(check_cpu_mode)
    mrs     x0, CurrentEL
    lsr     x0, x0, #2
    and     x0, x0, #3
    cmp     x0, #2
    b.eq    clear_bss
1:  wfe
    b       1b
END(check_cpu_mode)

FUNC_LOCAL(clear_bss)
    adrp    x0, __bss_start
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end
    add     x1, x1, :lo12:__bss_end
.Lclear_qword:
    sub     x2, x1, x0
    cmp     x2, #8
    b.lt    .Lclear_tail
    str     xzr, [x0], #8
    b       .Lclear_qword
.Lclear_tail:
    cbz     x2, .Lclear_done
    strb    wzr, [x0], #1
    sub     x2, x2, #1
    b       .Lclear_tail
.Lclear_done:
END(clear_bss)

FUNC_LOCAL(cpu_init)
    // Necessary after warm reset or hand-off from prior firmware.
    tlbi    alle2
    dsb     sy
    isb
    ic      iallu
    dsb     sy
    isb

    ldr     x0, =MAIRVAL
    msr     MAIR_EL2, x0
    ldr     x0, =(TCR_EL2_T0SZ(16) | TCR_EL2_IRGN0_WBWA | TCR_EL2_ORGN0_WBWA | TCR_EL2_SH0_IS   | TCR_EL2_TG0_4K     | TCR_EL2_PS_36)
    msr     TCR_EL2, x0
    isb

    msr     SPSel, #1

    // Reset value has CPTR_EL2.TFP=1; any FP instruction from Rust would fault without this.
    msr     cptr_el2, xzr
    isb

    // HCR_EL2 configuration:
    //   RW   (bit 31) — EL1 runs in AArch64, not AArch32
    //   AMO  (bit  5) — route physical SError to EL2
    //   IMO  (bit  4) — route physical IRQ to EL2
    //   FMO  (bit  3) — route physical FIQ to EL2
    mov     x0, #(HCR_EL2_FMO | HCR_EL2_IMO | HCR_EL2_AMO)
    movk    x0, #(HCR_EL2_RW >> 16), lsl #16
    msr     hcr_el2, x0
    isb

    // Required by any future guest OS that accesses the physical timer directly.
    mov     x0, #(CNTHCTL_EL2_EL1PCTEN | CNTHCTL_EL2_EL1PCEN)
    msr     cnthctl_el2, x0
    isb

    ldr lr, =init_stack
    b enable_boot_cpu_mm
END(cpu_init)

FUNC_LOCAL(init_stack)
    adrp    x0, stack_top
    add     x0, x0, :lo12:stack_top
    mov     sp, x0
END(init_stack)

FUNC_LOCAL(init_vector_table)
    adrp    x0, vector_table_el2
    add     x0, x0, :lo12:vector_table_el2
    msr     vbar_el2, x0
    dsb     sy
    isb
END(init_vector_table)

FUNC_LOCAL(launch)
    mov     x0, x20
    bl      main
END(launch)

FUNC(init_secondary)
    wfe
    b       init_secondary
END(init_secondary)

