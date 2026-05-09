// Exception entry/exit stubs for EL2.
//
// Each active exception class gets a thin assembly wrapper that:
//   1. Saves the complete register state to the stack (ExceptionFrame)
//   2. Calls the Rust handler via bl
//   3. Restores state and issues eret
//
// The eret lives here, never inside a Rust handler. Putting it in Rust means
// the compiler's function epilogue (ldp x29, x30 / ret) runs after eret fires,
// corrupting the stack pointer with a phantom frame.
//
// This file is coupled to vectors.s: every label exported here must match a
// branch target there. The split is intentional — vectors.s stays declarative,
// this file handles the mechanics.

.section .text.exception, "ax"

// ExceptionFrame layout — offsets must match the Rust struct in exceptions.rs.
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

// save_exception_frame — build an ExceptionFrame on the current stack.
//
// x9, x10, x11 are reused as scratch after their GPR values are in memory.
// This is safe: the original values are already at sp+72/80/88 when we read
// the system registers into those same hardware registers.
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

// restore_exception_frame — inverse of save_exception_frame.
//
// System registers are written before GPRs so x9/x10 can act as scratch here
// too, without clobbering the memory values we're about to load back into them.
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

// Synchronous exception taken while in EL2 (fault in hypervisor code).
// The Rust handler panics and never returns; restore + eret are unreachable.
// They are present so the stack is balanced if the handler is ever changed.
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
