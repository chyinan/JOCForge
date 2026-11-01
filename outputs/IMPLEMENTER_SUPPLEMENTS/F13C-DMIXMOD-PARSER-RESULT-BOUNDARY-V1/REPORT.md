# F13C `dmixmod` parser/result boundary

## Static finding

The hash-bound Ghidra trace shows `FUN_1404ecea0` reading four mix-level names and `dmixmod` separately. `dmixmod` passes through `FUN_1404dddd0` and is stored at internal slot `0x11`. The presence byte is checked; when the field is absent, conversion is skipped. The resulting native value/default remains `UNBOUND`.

The caller reaches this parser through `FUN_1403f4e90` → `FUN_1404802e0`. On the recorded `cVar3 == 0` branch, `FUN_1404802e0` calls `FUN_1404807d0`, which builds generic `name:`, `value:`, and `error:` result records. The available targeted body has no explicit downmix-key lookup, and the result-record identity relative to slot `0x11` remains `UNBOUND`.

## Consumer limit

No direct edge was established from the parser slot or generic result to a DDP JOC/E-AC-3 frame writer. This trace does not determine whether `dmixmod` controls stereo postmix, a downstream preferred-downmix signal, target 5.1 JOC processing/signaling, or multiple behaviors. The front-end `preferred_stereo_downmix` display value `Stereo` is inactive UI state and is not a proven native default or alias for `dmixmod`.

The imported image hash is historical Ghidra project metadata; the current installation binary was not rehashed. The thread's v3 package contained no verifier script. Wash manually recomputed its manifest-referenced file hashes, the listed checksum file, and the two referenced Ghidra log hashes, then checked the stated call/field edges. Wash v2 manifest `7dabde419bf0a5e378d94613393686298c2baff4053f015d9d09222e4e9fad4e` was independently confirmed in the implementation supplement directory.

## Disposition

Reference only; `implementation_ready=false`. Keep `preferred_downmix_mode`→`dmixmod`, omitted-field default, slot `0x11`→generic result identity, and slot `0x11`→audio writer/consumer all `UNBOUND`. No audio rule, default value, stereo branch, or 5.1 JOC effect is promoted. No encoder was launched, no PCM/payload/audio output was read or created, and no private key/license/HMAC content was read or exported.
