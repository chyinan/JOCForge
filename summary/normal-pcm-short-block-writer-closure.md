# Normal PCM-derived short-block writer closure

Verified 2026-09-08 against JOCForge start HEAD `bc0db9541fa6a30b1f99dda5e8c931f212cdc09d`, local OpenJOC `77e842303d4cb9d95b29ff41a3b9ed230f246b80`, ETSI TS 102 366 V1.4.1, and ETSI TS 103 420 V1.2.1.

## Audit

The normal CLI previously read each BW64 programme interval as a fixed 1,536-sample frame, encoded the complete I0 carrier once through FFmpeg, read fixed-size six-block frames, and packaged one I0 plus one D0. The existing `conformance-short` path emitted synthetic 512-byte frames and did not share the normal source-derived loop.

The pinned/local FFmpeg E-AC-3 encoder exposes no block-count option. Encoding 256, 512, or 768 input samples still produces a six-block 1,536-sample frame. The implementation therefore keeps FFmpeg as the default `[6]` core path and uses the existing public-syntax short-frame writer with source-carrier-derived bounded fields for the advanced raw strict short path. No six-block payload header is patched.

## Contracts

- ETSI Table E.1.3 maps `numblkscod` to 1/2/3/6 blocks, each 256 samples.
- A grouped normal JOC interval remains six blocks/1,536 samples.
- Central plan closure admits `6`, `3+3`, `2+2+2`, and `1+1+1+1+1+1` for strict raw ETSI I0+D0. Mixed six-block compositions remain `PUBLIC_AMBIGUOUS` in the normal plan; the pre-existing lower-level conformance fixture remains for regression evidence.
- Only final D0 owns JOC/OAMD EMDF. `convsync` is true only on the first short I0 in each group.
- Short raw writing is no-LFE and single-dependent only. LFE short framing, deployed short signaling, legacy-core short framing, and multi-dependent short combinations fail closed. The default six-block path retains existing LFE/deployed/legacy behavior.
- `partition_frame_bytes` distributes each six-block component budget in even 16-bit words, preserving the total frame-byte budget.

## Changed boundaries

- `conformance.rs` owns partition classification, exact sample ranges, short AU structural validation, and block timing.
- `plan.rs` carries `ShortBlockPlan` in `JocEncodingPlan`; short partitions are rejected for non-strict raw carriage before ADM I/O.
- `writer.rs` packages source-derived short groups from the normal factorization carriers and validates capacity before final padding/CRC2.
- `main.rs` parses `--block-partition`, keeps default six-block FFmpeg encoding, routes canonical short partitions through the source-derived writer, emits `.sample-accounting.csv`, removes stale outputs on plan/publish failure, and preserves CMAF's separate six-block path.
- The short closure script runs both synthetic conformance and real source-derived fixture output for idx0..idx4, with OpenJOC inspect/render, FFmpeg decode, accounting, and deterministic repeat checks.

## Evidence

The ignored `artifacts/short-block-closure/validation.json` contains 15 normal source-derived cells (five profiles × three short families) at strict 1664 kbps and 20 synthetic rows including the mixed fixture. The real cells each have 32 OpenJOC access units at 1,536 samples, zero malformed EMDF, valid final-D0 ownership, valid sample accounting, deterministic repeated output, successful OpenJOC render, and successful FFmpeg decode.
