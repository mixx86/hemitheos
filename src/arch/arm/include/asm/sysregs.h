#ifndef __ASM_SYSREGS_H__
#define __ASM_SYSREGS_H__

/*
 * TCR_EL2 — Translation Control Register (EL2)
 * ARMv8-A Architecture Reference Manual, D13.2.131
 */
#define TCR_EL2_T0SZ(x) ((x) & 0x3f) /* VA size: 64 - T0SZ bits */
#define TCR_EL2_IRGN0_WBWA (1 << 8)  /* Inner WB WA cacheable   */
#define TCR_EL2_ORGN0_WBWA (1 << 10) /* Outer WB WA cacheable   */
#define TCR_EL2_SH0_IS (3 << 12)     /* Inner Shareable         */
#define TCR_EL2_TG0_4K (0 << 14)     /* 4 KiB granule           */
#define TCR_EL2_PS_36 (1 << 16)      /* PA size: 36-bit (64 GB) */
#define TCR_EL2_PS_40 (2 << 16)      /* PA size: 40-bit (1 TB)  */
#define TCR_EL2_PS_48 (5 << 16)      /* PA size: 48-bit (256 TB)*/

/*
 * HCR_EL2 — Hypervisor Configuration Register
 * ARMv8-A Architecture Reference Manual, D13.2.46
 */
#define HCR_EL2_FMO (1 << 3) /* Route FIQ to EL2        */
#define HCR_EL2_IMO (1 << 4) /* Route IRQ to EL2        */
#define HCR_EL2_AMO (1 << 5) /* Route SError to EL2     */
#define HCR_EL2_RW (1 << 31) /* EL1 runs in AArch64     */

/*
 * SCTLR_EL2 — System Control Register (EL2)
 * ARMv8-A Architecture Reference Manual, D13.2.113
 */
#define SCTLR_EL2_M (1 << 0)  /* MMU enable              */
#define SCTLR_EL2_C (1 << 2)  /* Data cache enable       */
#define SCTLR_EL2_I (1 << 12) /* Instruction cache enable*/

/*
 * CNTHCTL_EL2 — Counter-timer Hypervisor Control Register
 * ARMv8-A Architecture Reference Manual, D13.11.5
 */
#define CNTHCTL_EL2_EL1PCTEN (1 << 0) /* Allow EL1 phys cnt access */
#define CNTHCTL_EL2_EL1PCEN (1 << 1)  /* Allow EL1 phys timer access */

#endif /* __ASM_SYSREGS_H__ */
