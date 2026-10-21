#ifndef __ARM_MMU_LAYOUT_H
#define __ARM_MMU_LAYOUT_H

#include <asm/config.h>
#include <macros.h>

#define FIXMAP_VIRT_START (HEMITHEOS_VIRT_START + HEMITHEOS_VIRT_SIZE)
#define FIXMAP_ADDR(n) (FIXMAP_VIRT_START + (n) * KB(4))

#endif
