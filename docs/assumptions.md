# Assumptions

- ARMv8-A AArch64, entry at Non-secure EL2 (otherwise the boot path halts)
- Bootloader follows the ARM64 Image protocol; firmware passes an argument in `x0`
- Platform discovery can supply RAM and CPU topology; a serial console is optional
