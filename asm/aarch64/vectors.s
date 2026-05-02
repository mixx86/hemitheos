.section .text.exception, "ax"
.global vector_table_el2

.p2align 11
vector_table_el2:
    // Group 1
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

    // Group 2
    .p2align 7
    b el2_sync_handler
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

    // Group 3
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .

    // Group 4
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
    .p2align 7
    b .
