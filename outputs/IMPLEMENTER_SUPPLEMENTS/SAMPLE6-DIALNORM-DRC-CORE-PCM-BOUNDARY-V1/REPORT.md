# Sample 6 DialNorm/DRC boundary for the 5.1 core PCM mismatch

Hash-bound Ghidra 12.2 evidence traces DDPi parameter ID `0x12` through a typed encoder-preparation path: per-stream `+0x9c` → nested context `+0x34` → word `+0x19140` → per-frame preparation record. The final join from that record to the E-AC-3 BSI U5 writer is still open. The bounded trace did not find the value feeding core PCM samples or coefficients.

The scoped 4-second job template reports DRC `none`, Lo/Ro, center level `-3`, and matching auto-trim. This is configuration evidence only. No DRC-specific field-to-PCM or field-to-BSI path was closed; stream `+0xa8` remains semantically unbound and must not be called DRC.

The six-value vector in this package is illustrative, not a DEE measurement. It distinguishes a signaling-only DialNorm hypothesis from a hypothetical pre-encode gain; it does not establish that DEE behaves either way. Thus this package does not explain the measured 5.1 core PCM residual and is not a basis for changing audio samples. Keep the field boundary available for a focused trial, but do not claim a BSI-only or PCM-invariant result.

No DEE rerun, PCM/payload read, key/license read, or output comparison occurred in this slice. The source executable identity is reported by the package; Wash did not locate or independently rehash the executable in this pass. No source code, decompilation, key, PCM, or encoded payload is included here.
