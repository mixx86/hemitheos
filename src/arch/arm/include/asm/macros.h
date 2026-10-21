.macro adr_l, dst, sym
  adrp \dst, \sym
  add  \dst, \dst, :lo12:\sym
.endm
