#ifndef __LINKAGE_H__
#define __LINKAGE_H__

/*
 * LASTARG(DEFAULT, OVERRIDE...) — picks the last argument.
 * Lets callers write FUNC(name) to use FUNC_ALIGN, or FUNC(name, 64)
 * to override, without two separate macros.
 */
#define count_args_(dot, a1, a2, a3, a4, x, ...) x
#define count_args(args...)     count_args_(., ## args, 4, 3, 2, 1, 0)
#define ARG1_(x, y...)          (x)
#define ARG2_(x, y...)          ARG1_(y)
#define ARG__(nr)               ARG ## nr ## _
#define ARG_(nr)                ARG__(nr)
#define LASTARG(x, y...)        ARG_(count_args(x, ## y))(x, ## y)

/* Default alignments (bytes). Override in arch headers before including this. */
#ifndef FUNC_ALIGN
# define FUNC_ALIGN 0
#endif
#ifndef DATA_ALIGN
# define DATA_ALIGN 0
#endif

#define SYM_L_GLOBAL(name)      .globl name; .hidden name
#define SYM_L_ENTRY(name)       .globl name
#define SYM_L_LOCAL(name)       /* local: no directive needed */

#define SYM_T_FUNC              STT_FUNC
#define SYM_T_DATA              STT_OBJECT
#define SYM_T_NONE              STT_NOTYPE

#define SYM(name, typ, linkage, align)  \
        .type name, SYM_T_ ## typ;     \
        SYM_L_ ## linkage(name);       \
        .balign align;                 \
        name:

#define END(name)               .size name, . - name

#define FUNC(name, align...)        SYM(name, FUNC, GLOBAL, LASTARG(FUNC_ALIGN, ## align))
#define FUNC_ENTRY(name, align...)  SYM(name, FUNC, ENTRY,  LASTARG(FUNC_ALIGN, ## align))
#define FUNC_LOCAL(name, align...)  SYM(name, FUNC, LOCAL,  LASTARG(FUNC_ALIGN, ## align))
#define LABEL(name, align...)       SYM(name, NONE, GLOBAL, LASTARG(FUNC_ALIGN, ## align))
#define LABEL_LOCAL(name, align...) SYM(name, NONE, LOCAL,  LASTARG(FUNC_ALIGN, ## align))
#define DATA(name, align...)        SYM(name, DATA, GLOBAL, LASTARG(DATA_ALIGN, ## align))
#define DATA_LOCAL(name, align...)  SYM(name, DATA, LOCAL,  LASTARG(DATA_ALIGN, ## align))

#endif /* __LINKAGE_H__ */
