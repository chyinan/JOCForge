# F13C dMixMod owned-consumer boundary

**Status:** parser owner bound; helper-result slot `+0x1b0` untyped; native audio/BSI/JOC effect unbound; A/B not run.

## Finding

The Ghidra 12.2 addendum binds the callback object's constructor vptr and parser handler: `FUN_1403f1dd0` installs vtable `0x140bcf690`, whose `+0xa8` slot is `FUN_1403f4e90`. The parser helper-result is carried at object `+0x1b0`, but its semantic type remains unbound.

Other accesses on that object do not close the gap. Slots `+0x28` and `+0x38` read doubles at `+0x58` and `+0x60`; slot `+0xb0` returns the subobject at `+0x70`. `FUN_1404813f0` consumes those separate values and writes derived fields at `+0xf0/+0xf4/+0xf8`; the traced path does not read `+0x1b0`. The per-item child path passes a separate item pointer and copies that item's fields; it does not consume the callback helper-result either.

The unique remaining edge is:

`callback object +0x1b0 helper-result → typed dMixMod owner/usepoint → BSI/side-info writer or separate renderer`

Until that edge or a discriminating experiment closes, LoRo/LtRt must remain **UNBOUND** as to whether it changes coded audio, changes only signaling, or is ignored on this path. Do not infer behavior from field names, neighboring fields, or matching numeric values.

## Controlled input-only plan

The source package contains one same-ADM A/B plan: use the same ADM identity and reconstructed job template, changing only `preferred_downmix_mode` from `loro` to `ltrt`, while holding all four mix-level fields and all other job controls fixed. The template was reconstructed from source and is not a captured runtime XML. The comparison is **PREPARED_NOT_RUN**. Its own gate says to wait until the current R4 DEE/WinDbg observation ends; at Wash review time R4 was still active. No output, BSI, decoded PCM, or audible-effect result is claimed.

## Evidence scope

Wash independently ran the source package verifier in normal Python and `python -O`; both reported 9 payloads and 10 checksums valid. The source package includes `SHA256SUMS.txt` covering `MANIFEST.json`, but has no separate `package_seal.json`. The Ghidra record reports that the installed image hash matched the project-recorded image; Wash did not independently rehash that binary or rerun Ghidra. The reported analysis is static only. No DEE, WinDbg, media, PCM, encoded payload, private key, or license content was accessed for this addendum.

This is a bounded evidence supplement, not an implementation instruction to invent native semantics. The selected XML option's native consumer and audible effect remain open.
