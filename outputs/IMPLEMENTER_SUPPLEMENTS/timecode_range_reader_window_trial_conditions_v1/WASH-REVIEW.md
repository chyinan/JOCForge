# Wash review: full-song MVP trial conditions

Disposition: evidence-only trial profile, not product conformance or audio-quality acceptance.

Use the user-authorized sample-6 input for one full-file JOCForge attempt. Rehash the original first, keep it unchanged, and use a separate ASCII-named copy if needed; verify that copy SHA-256 matches. Derive the complete extent from the actual BWF header.

The existing user-supplied DEE full-song transcript is not separately hash-bound. It reports `atmos_mezz`, `encode_to_atmos_ddp`, `file_position`, empty start/end, 1024 kbps, 48 kHz, source count 6,522,906, offset=FFOA=3610 seconds, 24 fps, and Composite Master start/end 0. Treat these as trial cross-checks, not an authenticated same-run source join. The hash-bound Ghidra branch evaluates `max(offset, FFOA)-offset`; for the logged equal pair it yields zero. Do not turn that one case into a universal rule.

R3's separate `audio.adm` embedded-timecode runs on two distinct inputs report frames 48000-97999 for tokens 86664-86688. That is a temporary exact-input/config/token observation; it does not define the full-song file_position route.

Three R3 receipts are hash-checked. First Love is a quiet-window exclusion; Hortus and My Jealousy are active-window receipts. FFprobe/OpenJOC are posthoc format/signaling checks only, not DEE algorithm, default, numeric-field, or policy sources. Output hashes remain receipt claims and were not rehashed by Wash. No payload, PCM, decode, rendering, listening, or full-track JOCForge validation is included.

Audio-affecting behavior remains limited to DEE observations and separately approved Wash specifications. This package authorizes only use of the recorded input/profile conditions for the MVP trial.
