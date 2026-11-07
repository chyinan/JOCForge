# Silence builder call and first post-call boundary

Reference-only Wash supplement; it establishes a bounded callsite/dataflow and a diagnostic first use, not silence insertion or audio effect. `IMPLEMENTATION_READY=NO`.

At callsite `FUN_180175c80 → FUN_1801844c0` (`0x175e10`), the caller passes its output vector in RCX. R8 comes from the per-source context `+0x18` prepend-count vector; stack arg6 comes from context `+0x60` append-count vector. Other parallel vectors come from context `+0x30` and `+0x48`. The callee's returned pointer is not assigned.

After return, the first statement copies a logger interface into a local. The first subsequent use of the composite-row output vector is `FUN_1801849c0` at `0x175ea6`; its result is formatted into an “Initializing Composite Master” message and sent to a logger callback. This is the observed first post-call use. `FUN_1801849c0` was not expanded, so whether it reads row fields remains `UNBOUND`. No link to Reader, sample insertion, PTS, or cleanup was established here.