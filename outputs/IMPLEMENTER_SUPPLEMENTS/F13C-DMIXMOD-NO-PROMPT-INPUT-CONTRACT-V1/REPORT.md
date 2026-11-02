# F13C downmix input contract: no user prompt

## MVP input rule

Do not ask the user to choose Lo/Ro or Lt/Rt. The seven authorized ADM/BWF metadata scans found no preferred-downmix AXML field, and DME's optional control is inactive by default and omitted by its request builder. For the selected attempt-10 JOCForge job template, keep the explicit generated `loro` value with four `-3 dB` levels; the job parser requires those fields.

The current hash-verified Rust source path maps that configuration to BSI `dmixmod=2` and four level codes `4`. These are source-path values, not a runtime observation of attempt-10's final frame bits.

## Native boundary and confidence

Ghidra 12.2 traces native `dmixmod` parsing/validation to `FUN_1404807d0`, whose body is a generic name/value/error result-tree builder. The traced body has no downmix or mix-level reads and is not an identified DSP/postmix consumer. The native validator strings (`not_indicated`, `pro_logic`, `stereo`, `pro_logic_ii`) are not typed enum values; `loro → native ddp_dmixmod`, its audio effect, and the effective native default on omission remain `UNBOUND`.

Confidence is high for the no-prompt input contract and source-generated attempt-10 template; native DSP semantics are not established. This uncertainty does not block the explicit job input rule and does not justify inventing a user option. The current installed Ghidra target image was project-recorded but not rehashed in this addendum.

## Implementation disposition

Preserve the existing `loro/-3×4` template for the selected attempt-10 path, preserve omission when the optional DME input is inactive, and avoid prompting. Keep this config seam separate from the JOC audio-transform path. If a source-only fixture is useful, generate the selected XML and verify parser-to-BSI mapping; no encoder or media run is needed for that contract.

## Exclusions

No implementation source, DEE binary, disassembly, decompilation, media, PCM, private key, or license material is included. The source tree is bound only by hashes.
