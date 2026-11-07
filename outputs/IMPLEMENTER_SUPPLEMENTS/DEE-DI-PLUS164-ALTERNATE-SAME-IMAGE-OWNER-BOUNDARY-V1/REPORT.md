# Alternate same-image +0x164 reader owner boundary

One alternate function in the same DDP Atmos image reads an integer from its param_1 +0x164. Its single inspected caller supplies a pointer from a codec-parent child record. The established EmpireLoudness result path supplies a different subcontext pointer from engine +0x1d0.

No typed pointer/owner edge joins the codec-parent child record to the EmpireLoudness engine subcontext. This candidate therefore does not change the Bool-to-result boundary: causality remains UNBOUND. Only one candidate and one caller were checked; no global absence claim or runtime observation is made.
