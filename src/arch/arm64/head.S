// Atomic flag used to elect the primary CPU at boot.
// Lives in .data so it is zero when the image is loaded, before BSS is cleared.
.section .data
.align 2
_boot_cpu_online:
    .word 0

.section .text.boot
.global _img_start
.global _start

// ARM64 Image header (boot protocol)
_img_start:
    b       _start          // skip image header
    .word   0               // code1
    .quad   0x80000         // text_offset: load at RAM_BASE + 0x80000
    .quad   0               // image_size: 0 = use file size
    .quad   0               // flags
    .quad   0               // reserved
    .quad   0               // reserved
    .quad   0               // reserved
    .word   0x644d5241      // magic: ARM64 Image marker ("ARMd")
    .word   0               // res5

_start:
    msr DAIFSet, #0xf

    // Elect the primary CPU via atomic exchange.
    // The first CPU to claim _boot_cpu_online proceeds; all others are parked.
    // This is topology-agnostic: works regardless of MPIDR values, big.LITTLE,
    // or cluster layout. On PSCI systems secondaries never reach here anyway.
    adrp    x1, _boot_cpu_online
    add     x1, x1, :lo12:_boot_cpu_online
    mov     w2, #1
.Lclaim:
    ldaxr   w3, [x1]               // load-acquire exclusive
    cbnz    w3, secondary_halt     // already claimed — we are secondary
    stlxr   w4, w2, [x1]           // store-release exclusive; w4=0 on success
    cbnz    w4, .Lclaim            // lost exclusive monitor — retry

    // Save firmware boot argument (passed in x0) before stack/BSS setup
    mov     x20, x0                // preserve boot_arg (x20 is callee-saved ABI register)

    mrs     x0, CurrentEL
    lsr     x0, x0, #2
    and     x0, x0, #3
 
    cmp     x0, #2
    b.eq    el2_entry

    b       unsupported_el

el2_entry:
    msr     SPSel, #1

    // Invalidate stale TLB entries and instruction cache.
    // Necessary on real hardware after warm reset or hand-off from prior firmware.
    tlbi    alle2
    dsb     sy
    isb
    ic      iallu
    dsb     sy
    isb

    // Disable FP/SIMD/SVE/SME trapping to EL2.
    // Reset value has TFP=1; any FP instruction from Rust would fault without this.
    msr     cptr_el2, xzr
    isb

    // Configure HCR_EL2:
    //   RW   (bit 31) — EL1 runs in AArch64, not AArch32
    //   AMO  (bit  5) — route physical SError to EL2
    //   IMO  (bit  4) — route physical IRQ to EL2
    //   FMO  (bit  3) — route physical FIQ to EL2
    mov     x0, #0x38               // AMO | IMO | FMO
    movk    x0, #0x8000, lsl #16    // RW (bit 31)
    msr     hcr_el2, x0
    isb

    // Allow EL1 to access the physical timer counter and control registers
    // without trapping to EL2 (required by any future guest OS).
    mov     x0, #3
    msr     cnthctl_el2, x0
    isb

    adrp    x0, stack_top
    add     x0, x0, :lo12:stack_top
    mov     sp, x0

    adrp    x0, vector_table_el2
    add     x0, x0, :lo12:vector_table_el2
    msr     vbar_el2, x0
    dsb     sy
    isb

    adrp    x0, __bss_start
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end
    add     x1, x1, :lo12:__bss_end
clear_bss_qword:
    sub     x2, x1, x0
    cmp     x2, #8
    b.lt    clear_bss_tail
    str     xzr, [x0], #8
    b       clear_bss_qword
clear_bss_tail:
    cbz     x2, clear_bss_done
    strb    wzr, [x0], #1
    sub     x2, x2, #1
    b       clear_bss_tail

clear_bss_done:
    mov     x0, x20
    bl      main

// park secondary cores indefinitely
secondary_halt:
    wfe
    b       secondary_halt

// unsupported exception level (should never happen)
unsupported_el:
    wfe
    b       unsupported_el
