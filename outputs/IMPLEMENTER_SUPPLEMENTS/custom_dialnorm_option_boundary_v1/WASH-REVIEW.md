# custom_dialnorm option boundary

## Closed static field

Ghidra binds the DDP option key custom_dialnorm to an Int64 SafeType with inclusive range -31..0 and default zero. The DDP option description says zero disables override of measured loudness. A nonzero value converts to Float32 and overwrites three encode-parameter record fields. The separate host key loudness_target is not this option.

## Remaining boundaries

The static databases do not type-join host XML parsing to the DDP option instance. The run144 task XML omitted custom_dialnorm; the runtime option value and override branch were not captured. The conditional Float32 parameter overwrite has not been joined to the DDP U16 field consumed by the 5-bit serializer. Run144 logged measured loudness -27.774199 (unit unstated) and observed U16 28 in the same owner consumer chain; this numeric match does not prove a formula.

## Wash disposition

For the run144 XML, do not invent a user-selected custom_dialnorm value. Keep its documented default-zero semantics unless the XML supplies the key. Do not derive the U16 DialNorm field from the measurement based on the observed point. A same-job capture plan is already attached to the R3 one-scalar run and starts no additional DEE job.
