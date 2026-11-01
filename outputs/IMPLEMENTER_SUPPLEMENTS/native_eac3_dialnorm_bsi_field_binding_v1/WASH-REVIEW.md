# E-AC-3 DialNorm BSI field binding

## Field identity

Ghidra binds one same-owner U16 field at +0x34 through a setter copy to staging and then to the serializer record. The serializer writes the record member at +0x20 as a five-bit BSI field. The surrounding header order places BSID immediately before it and compre immediately after it, so this field is E-AC-3 DialNorm at logical bits 45–49.

Run-144 independently captured U16 28 through the same-owner setter/staging path and as U5 28 at the serializer. The output payload was not reopened for bit-level verification in this observation.

## Producer remains open

Run-144 reported integrated loudness -27.774199 (unit unstated), with custom_dialnorm absent from the task XML. A separate static package binds custom_dialnorm default zero to “no override,” but no typed path from the F06 Float32 measurement/result to the DialNorm U16 producer was found. The numeric match does not establish the formula, rounding, clamp, or causal generation.

## Wash disposition

The field may now be named DialNorm in the implementation map. Its producer and value rule remain unapproved. Do not calculate it from measured loudness using the observed sample alone.

## Provenance

Source static package manifest SHA-256: cdac36b418fddeb8e587894622d4c6019b9909b632ff4437e8a97c54fde3d3a0
Run-144 source package manifest SHA-256: fe3ed5df47b60ea4861a314225ef14aaa4f2d2398f5b59a2c30e35b6c022ae64
Both source validators passed in normal and optimized modes. This Wash package contains no binary, source code, disassembly, private key, PCM, or raw media.
