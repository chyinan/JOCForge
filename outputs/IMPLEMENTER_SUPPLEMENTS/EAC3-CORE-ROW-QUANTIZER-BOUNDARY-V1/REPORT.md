# E-AC-3 core transform-row to BAP/mantissa boundary

R2's Ghidra 12.2 static slice binds a shared typed row-pointer table from the native transform writers to the BAP power consumer and the mantissa coefficient input. The matching Rust paths use the corresponding primary/companion rows. This is static pointer-base/index evidence; no live pointer generation, physical source-channel mapping, or row-value identity for sample 6 was observed.

For one synthetic row, native and Rust share the row-power rule `min(f32(primary² + companion²), 1.0)`. With primary `0.25`, companion `0.5`, exponent `0`, and BAP `1`, both models produce power `0.3125` (`0x3ea00000`) and BAP-1 scalar code `1` at width 5. The BAP-1 scalar formula uses offset `1.0`, scale `1.5`, truncation and clamp to `[0,2]`. Grouped mantissa packing is not exercised by this vector.

This narrow discriminator found no numeric split in the checked formulas. It does not clear sample-6 parity: the actual row values, BAP producer, physical row/lane identity and old run's source snapshot remain unbound. It does not explain the PCM mismatch. The next useful edge is the native quantizer call's coefficient-row pointer versus BAP-row/index identity, followed to the transform row writers.

The current provider pin and current 256-word window table both match the DEE candidate after the 12-word trial edit. Whether the historical sample-6 PCM run used that source snapshot is not established here. No DEE run, PCM/payload read, or private-key access occurred; no raw code/decompilation is included.
