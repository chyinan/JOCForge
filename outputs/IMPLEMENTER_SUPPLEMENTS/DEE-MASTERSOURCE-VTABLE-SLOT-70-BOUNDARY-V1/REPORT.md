# MasterSource indirect slot +0x70 boundary

Reference-only Wash addendum. It confirms the indirect-dispatch form, but not the target function or field semantics. `IMPLEMENTATION_READY=NO`.

In the bounded `FUN_1804cba30` body, `param_1` is represented as `longlong *`. At `0x1804cba3d` the function loads the first qword as a vtable pointer; at `0x1804cba40` it calls the pointer at vtable `+0x70` without explicit arguments. The current function view has no concrete vtable base or typed owner, and no callsite/xref scan was made.

The first break is concrete `param_1` owner/vptr/generation → slot target address. Do not infer that `MasterSourceImpl+0x20/+0x28` are start/end samples, and do not infer a range, duration, PTS, or audio effect.