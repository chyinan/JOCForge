# R138 MediaMetadata row writer and ADM identity crosswalk

## Best-supported MVP rule

For an ordered converter source vector, preserve its row order when building the `MediaMetadata` vector. For zero-based source row ordinal `r`, write metadata `index = r + 1`, copy the source filename, and preserve the nested string key/value fields. Keep converter identities typed and separate:

- `AO_%04x` suffix → zero-based `ConverterChannelInputId` by subtracting `0x1001`.
- `ATU_%08x` suffix → zero-based CHNA UID-table ordinal by subtracting `1`.
- The converter checks decoded IDs against the companion source-vector row at the same loop ordinal after verifying equal row counts.

The selected process-state `+0xA0` member is a `dee::Types::MediaMetadata` SafeType whose payload is `vector<map<string,string>>`. The type-specific setter constructs one output map per input row and commits the vector through the SafeType value interface.

This is a usable input metadata model rule. Do not infer that the seven-file metadata scan order is the runtime source-vector order; do not join identities through equal scalar values. The exact selected R138 setter invocation and same-object owner/generation join remain `UNBOUND`, but are not a gate for trying the supported row-preserving rule.

## Coverage and limits

The metadata constraint covers seven 48 kHz authorized inputs: six 36-row CHNA inputs and one 38-row `人是猫` fixture (SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`). The seven-file scan constrains input metadata shape; it does not prove runtime vector identity or row-pointer ownership. No PCM or payload was read in this slice; no DEE run occurred.

Confidence is high for the type-specific row writer and converter-internal typed crosswalk, and medium for composing those rules into the model input. A wrong selected row could alter filename/sample/channel metadata consumed by the filter. Actual R138 output values and bitstream impact were not evaluated.

## Exclusions

No source code, DEE binary, disassembly, decompilation, audio, PCM, private key, or license material is included. Ghidra report hashes and target image hashes are summarized in `SOURCE-MANIFESTS.json`.
