# Atmos JOC inner preprocessor dispatch

This V2 supplement preserves the prior conditional rule and adds a
hash-bound call-level classification. The `SinglethreadSymphonyRenderer`
helper calls nested component `vtable+0x28`; the component is selected from a
configuration object's `vtable+0x38` result. The selector was not recorded in
the two authorized historical ADM receipts, so do not invent a default or
assign either historical run to a branch.

For selector zero, `SpatialCoderPlatform.vtable+0x28` routes through
`FUN_180443920` and `FUN_1804493d0` to a state/control helper, not a sample
transform. The sample-processing path is a separate slot:
`vtable+0x08 -> FUN_1804437d0 -> FUN_1804497c0`, which contains the
`sc_process` path and float-backed block work. For a nonzero selector,
`PreProcessorBypass` is constructed; its `+0x28` entry reaches a CFG guard
thunk, while the actual indirect target remains UNBOUND.

The practical MVP rule is conditional dispatch on the typed selector. The
missing selector value does not block that model path, but the package does
not establish which branch a particular ADM run uses or exact audio
equivalence. Static evidence only; no DEE run, PCM, payload, or key was read.
