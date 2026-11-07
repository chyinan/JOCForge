# Atmos preferred downmix property: native consumer boundary

## Bound input candidate

The reviewed candidate is an `atmos_mezz` job at 1024 kbps with the four numeric mix levels held at -3 dB. Its only declared XML difference is `preferred_downmix_mode`: candidate A is `loro`; candidate B is `ltrt`. The two recorded XML SHA-256 values are retained in `INPUT-ONLY-VECTORS.json`. Schema validation is recorded as passed in the upstream receipt; neither DEE candidate was run.

The upstream receipt associates the input alias `human_is_cat_ascii.wav` with the user-confirmed Human Cat ADM identity SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1` and length 743,696,526 bytes. That identity was not recomputed in this pass; no BWF content or PCM was read.

## One native boundary

The static Ghidra 12.2 receipt binds a distinct native key, `dmixmod`, to parser `FUN_1404ecea0`; its first proven text consumer, `FUN_1404dddd0`, checks presence and membership in `not_indicated`, `pro_logic`, `stereo`, and `pro_logic_ii`. No typed alias or owner connects the XML property `preferred_downmix_mode` (`loro`/`ltrt`) to that key or to a typed `ddp_dmixmod` member. The separate numeric mix-level path does not consume the preferred-mode enum in the bounded trace.

The precise first break is `preferred_downmix_mode` XML string → typed native `ddp_dmixmod` owner/consumer. Atmos 5.1/JOC audio, BSI/metadata, UI/reporting-only behavior, and intentional ignore all remain `UNBOUND`; spatial-object rendering is `NOT_COVERED`. No mode policy or audio mapping is approved by this receipt.

## Provenance and verification

The Wash-only upstream package manifest SHA-256 is `1c75cf415e566ccf260e031ef8bb818c293c7d32fcd04de28a78b29502c4e508`. Its normal and `python -O` checker both passed, verifying five payloads, six checksum entries, and seven parent receipts. The upstream directory contains no separate `package_seal.json`; its manifest is covered by `SHA256SUMS.txt`. The upstream checker and source manifests are not copied into this implementer supplement; only sanitized findings and hash-bound manifests are included.

No new Ghidra query, DEE run, A/B, WinDbg, media/PCM/payload/key/license read, or implementation contact occurred in the source pass. The receipt is evidence-only and must not be treated as a closed product behavior.
