.section .text.exception, "ax"
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
