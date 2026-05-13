// EL2 exception vector table and entry/exit stubs.
//
// Layout follows the ARMv8-A exception model: four groups of four vectors,
// each 128 bytes apart, 2 KiB-aligned. Only the EL2/SP_EL2 group is active;
// all other entries halt with `b .` until the corresponding phase implements them.
//
// Each active vector branches to an entry stub that:
//   1. Saves the complete register state to the stack (ExceptionFrame)
//   2. Calls the Rust handler via bl
//   3. Restores state and issues eret
//
// The eret lives here, never inside a Rust handler. Putting it in Rust means
// the compiler's function epilogue (ldp x29, x30 / ret) runs after eret fires,
// corrupting the stack pointer with a phantom frame.

.section .text.exception, "ax"

// ── Vector table ─────────────────────────────────────────────────────────────

.global vector_table_el2
.p2align 11
vector_table_el2:
    // EL2, SP_EL0 — unused; the hypervisor never switches to the task stack pointer
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

    // EL2, SP_EL2 — exceptions taken while running in the hypervisor itself
    .p2align 7
    b el2_sync_entry
    .p2align 7
    b el2_irq_entry
    .p2align 7
    b el2_fiq_entry
    .p2align 7
    b el2_serror_entry

    // Lower EL, AArch64 — exceptions from AArch64 guests (future: trap dispatch)
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

    // Lower EL, AArch32 — AArch32 guests not supported
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

// ── ExceptionFrame ────────────────────────────────────────────────────────────
//
//   Offset  Content
//   0       x0  .. x29  (consecutive pairs, 8 bytes each)
//   240     x30
//   248     elr_el2     (return address into interrupted code)
//   256     spsr_el2    (PSTATE to restore on eret)
//   264     esr_el2     (exception syndrome; read-only, saved for diagnostics)
//
// 272 bytes total, 16-byte aligned (required by AArch64 stack discipline).
.set FRAME_SIZE, 272

// x9, x10, x11 are reused as scratch after their GPR values are in memory.
// Safe because the originals are already at sp+72/80/88 when we overwrite those
// registers with system register reads.
.macro save_exception_frame
    sub  sp,  sp,  #FRAME_SIZE
    stp  x0,  x1,  [sp, #0]
    stp  x2,  x3,  [sp, #16]
    stp  x4,  x5,  [sp, #32]
    stp  x6,  x7,  [sp, #48]
    stp  x8,  x9,  [sp, #64]
    stp  x10, x11, [sp, #80]
    stp  x12, x13, [sp, #96]
    stp  x14, x15, [sp, #112]
    stp  x16, x17, [sp, #128]
    stp  x18, x19, [sp, #144]
    stp  x20, x21, [sp, #160]
    stp  x22, x23, [sp, #176]
    stp  x24, x25, [sp, #192]
    stp  x26, x27, [sp, #208]
    stp  x28, x29, [sp, #224]
    str  x30,      [sp, #240]
    mrs  x9,  elr_el2
    mrs  x10, spsr_el2
    mrs  x11, esr_el2
    stp  x9,  x10, [sp, #248]
    str  x11,      [sp, #264]
.endm

// System registers are written before GPRs so x9/x10 can serve as scratch
// without clobbering the values we are about to load back into them.
// esr_el2 is read-only; there is nothing to restore.
.macro restore_exception_frame
    ldp  x9,  x10, [sp, #248]
    msr  elr_el2,  x9
    msr  spsr_el2, x10
    ldr  x30,      [sp, #240]
    ldp  x28, x29, [sp, #224]
    ldp  x26, x27, [sp, #208]
    ldp  x24, x25, [sp, #192]
    ldp  x22, x23, [sp, #176]
    ldp  x20, x21, [sp, #160]
    ldp  x18, x19, [sp, #144]
    ldp  x16, x17, [sp, #128]
    ldp  x14, x15, [sp, #112]
    ldp  x12, x13, [sp, #96]
    ldp  x10, x11, [sp, #80]
    ldp  x8,  x9,  [sp, #64]
    ldp  x6,  x7,  [sp, #48]
    ldp  x4,  x5,  [sp, #32]
    ldp  x2,  x3,  [sp, #16]
    ldp  x0,  x1,  [sp, #0]
    add  sp,  sp,  #FRAME_SIZE
.endm

// ── Entry stubs ───────────────────────────────────────────────────────────────

// The Rust handler panics and never returns; restore + eret are unreachable.
// Kept for stack symmetry in case the handler is ever changed to return.
.global el2_sync_entry
el2_sync_entry:
    save_exception_frame
    bl   EL2_Sync_Handler
    restore_exception_frame
    eret

// IRQ from EL2. No GIC configured yet; any delivery at this stage is spurious.
.global el2_irq_entry
el2_irq_entry:
    save_exception_frame
    bl   EL2_IRQ_Handler
    restore_exception_frame
    eret

// FIQ from EL2. On most platforms FIQs are routed to EL3 and should not
// reach EL2 before a GIC handoff; treat as spurious for now.
.global el2_fiq_entry
el2_fiq_entry:
    save_exception_frame
    bl   EL2_FIQ_Handler
    restore_exception_frame
    eret

// SError (asynchronous abort — bus error, ECC fault, uncorrectable memory).
// Not recoverable; the Rust handler panics.
.global el2_serror_entry
el2_serror_entry:
    save_exception_frame
    bl   EL2_SError_Handler
    restore_exception_frame
    eret
