#define PAGE_SHIFT 12
#define PAGE_SIZE (1 << PAGE_SHIFT)
#define PT_INDEX_BITS 9
#define PT_LEVEL_SHIFT(lvl) (PAGE_SHIFT + PT_INDEX_BITS * (3 - (lvl)))

#define L0_INDEX(addr) (((addr) >> PT_LEVEL_SHIFT(0)) & 0x1FF)
#define L1_INDEX(addr) (((addr) >> PT_LEVEL_SHIFT(1)) & 0x1FF)
#define L2_INDEX(addr) (((addr) >> PT_LEVEL_SHIFT(2)) & 0x1FF)
#define L3_INDEX(addr) (((addr) >> PT_LEVEL_SHIFT(3)) & 0x1FF)

#define MT_DEVICE_nGnRnE 0x0
#define MT_NORMAL_NC 0x1
#define MT_NORMAL_WT 0x2
#define MT_NORMAL_WB 0x3
#define MT_DEVICE_nGnRE 0x4
#define MT_NORMAL 0x7

#define _MAIR_LOW(attr, mt) (attr << ((mt) * 8))
#define _MAIR_HIGH(attr, mt) (attr << (((mt) * 8) - 32))

#define MAIRLOWVAL                                                             \
  (_MAIR_LOW(0x00, MT_DEVICE_nGnRnE) | _MAIR_LOW(0x44, MT_NORMAL_NC) |         \
   _MAIR_LOW(0xaa, MT_NORMAL_WT) | _MAIR_LOW(0xee, MT_NORMAL_WB))

#define MAIRHIGHVAL                                                            \
  (_MAIR_HIGH(0x04, MT_DEVICE_nGnRE) | _MAIR_HIGH(0xff, MT_NORMAL))

#define MAIRVAL (MAIRHIGHVAL << 32 | MAIRLOWVAL)

/*
 * Layout of the flags used for updating the hypervisor page tables
 *
 * [0:2] Memory Attribute Index
 * [3:4] Permission flags
 * [5]   Page present
 * [6]   Only populate page tables
 * [7]   Superpage mappings is allowed
 * [8]   Set contiguous bit (internal flag)
 */
#define _PAGE_XN_BIT 3
#define _PAGE_RO_BIT 4

#define _PAGE_XN (1 << _PAGE_XN_BIT)
#define _PAGE_RO (1 << _PAGE_RO_BIT)

#define PAGE_XN_MASK(x) (((x) >> _PAGE_XN_BIT) & 0x1)
#define PAGE_RO_MASK(x) (((x) >> _PAGE_RO_BIT) & 0x1)

#define _PAGE_PRESENT (1 << 5)

#define _PAGE_CONTIG_BIT 8
#define _PAGE_CONTIG (1 << _PAGE_CONTIG_BIT)
