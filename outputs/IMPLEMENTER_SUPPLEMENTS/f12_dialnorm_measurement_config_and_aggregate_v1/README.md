# F12 measurement configuration and native scalar aggregation

In the sampled DME job, metering_mode is 1770-4 and is a measurement-method selector, not a user-supplied loudness scalar. The config loader maps the key to an internal selector slot; the value matches its default. The sample has measure_only present, dialogue_intelligence false, and no custom_dialnorm element.

The hash-bound native DDP measurement path initializes an aggregate float to -120.0, iterates a per-source result vector with 0x20-byte stride, reads a float at offset +0x14, and replaces the aggregate only when the candidate is strictly greater. Equal candidates retain the earlier aggregate. This supports a max-selection rule for the aggregate only; it does not specify the per-source audio measurement algorithm or the unit.

One authorized My jealousy ADM run observed native scalar -17.093924. The native log and static labels do not state its unit; meter-vtable unit conversion was not traced. There was no native rounding or quantization observed in the aggregate. The ADM AXML had no loudness-related field. PCM was not decoded or analyzed, and no E-AC-3 output was generated.
