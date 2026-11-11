#include <linkage.h>
#include <asm/page.h>
#include <asm/config.h>
#include <asm/mmu/layout.h>
#include <asm/macros.h>

#define PT_PT     0x03 /* T=1 P=1 */
#define PT_MEM    0xf7f   /* nG=1 AF=1 SH=11 AP=01 NS=1 ATTR=111 T=1 P=1 */
#define PT_MEM_2  0xf7d   /* nG=1 AF=1 SH=11 AP=01 NS=1 ATTR=111 T=0 P=1 */

/*
 * Macro to find the slot number at a given page-table level
 *
 * slot:     slot computed
 * virt:     virtual address
 * lvl:      page-table level
 */
.macro get_table_slot, slot, virt, lvl
        ubfx  \slot, \virt, #PT_LEVEL_SHIFT(\lvl), #PT_INDEX_BITS
.endm


FUNC(enable_boot_cpu_mm)
    b create_page_tables
END(enable_boot_cpu_mm)

FUNC_LOCAL(create_page_tables)
    ldr     x0, =HEMITHEOS_VIRT_START
    get_table_slot  x1, x0, 0
    get_table_slot  x2, x0, 1
    get_table_slot  x3, x0, 2

    adr   x19, start
    sub   x21, x19, x0        // x20 = phys_offset = PA - VA

    // L0 → L1
    ldr   x0, =boot_pgtable
    add   x0, x0, x21
    ldr   x4, =boot_first
    add   x4, x4, x21
    orr   x4, x4, #3
    str   x4, [x0, x1, lsl #3]

    // L1 → L2
    ldr   x0, =boot_first
    add   x0, x0, x21
    ldr   x4, =boot_second
    add   x4, x4, x21
    orr   x4, x4, #3
    str   x4, [x0, x2, lsl #3]

    // L2 → 2MB blocs
    ldr   x0, =boot_second
    add   x0, x0, x21
    mov   x4, x19
    orr   x4, x4, #(1 << 10)   // AF
    orr   x4, x4, #(3 << 2)    // AttrIndx = MT_NORMAL_WB
    orr   x4, x4, #1            // block descriptor
    str   x4, [x0, x3, lsl #3]

    ldr     x5, =HEMITHEOS_VIRT_START
    ldr     x6, =_end
    sub     x6, x6, x5
    ldr     x7, =((1 << 21) - 1)
    add     x6, x6, x7           // suma 2MB-1 antes de dividir
    lsr     x6, x6, #21          // redondea hacia arriba
    sub     x6, x6, #1

1:  cbz   x6, 2f
    add   x3, x3, #1
    add   x4, x4, #(1 << 21)
    str   x4, [x0, x3, lsl #3]
    sub   x6, x6, #1
    b     1b

2:  ldr     x0, =FIXMAP_VIRT_START
    get_table_slot x3, x0, 2

    // boot_fixmap -> L2
    ldr   x0, =boot_second
    add   x0, x0, x21
    ldr   x5, =boot_fixmap
    add   x5, x5, x21
    orr   x5, x5, #PT_PT
    str   x5, [x0, x3, lsl #3]

3:  adr x0, enable_mmu
    get_table_slot  x1, x0, 0
    get_table_slot  x2, x0, 1
    get_table_slot  x3, x0, 2
    get_table_slot  x4, x0, 3

    // L1 -> L0
    ldr   x0, =boot_pgtable
    add   x0, x0, x21
    ldr   x5, =boot_first_id
    add   x5, x5, x21
    orr   x5, x5, #PT_PT
    str   x5, [x0, x1, lsl #3]

    // L2 -> L1
    ldr   x0, =boot_first_id
    add   x0, x0, x21
    ldr   x5, =boot_second_id
    add   x5, x5, x21
    orr   x5, x5, #PT_PT
    str   x5, [x0, x2, lsl #3]

    // L3 -> L2
    ldr   x0, =boot_second_id
    add   x0, x0, x21
    ldr   x5, =boot_third
    add   x5, x5, x21
    orr   x5, x5, #PT_PT
    str   x5, [x0, x3, lsl #3]

    adr   x0, enable_mmu
    ldr   x5, =boot_third
    add   x5, x5, x21             // x5 = PA of L3 table (base for str)
    mov   x6, #PT_MEM
    orr   x0, x0, x6              // x0 = PA of page | attrs (entry value)
    str   x0, [x5, x4, lsl #3]
END(create_page_tables)

FUNC_LOCAL(enable_mmu)
    dsb     sy
    isb
    ldr   x0, =boot_pgtable
    add   x0, x0, x21
    msr   TTBR0_EL2, x0
    isb

    mrs   x0, sctlr_el2
    orr   x0, x0, #1
    msr   sctlr_el2, x0
    isb

    ret
END(enable_mmu)

