# Selected EMDF carrier and publication gate

This is a bounded F13C transport slice for the washed selected ordinary 48 kHz, 1024 kbps, one-independent-substream profile. It implements the six-block carrier planner, reservation bit field, fragment byte writer, and outer frontend publication gate. Whole F13C/F13 remains `IMPLEMENTATION_READY=NO`.

## Carrier planner and writer

The planner request is a signed I32 byte count for the externally supplied serialized body, excluding the four-byte sync/length prefix. Each call clears six segment lengths, advances the period-62 counter (wrapping exactly at 62), and updates the enabled flag. A nonpositive request preserves the prior first-block field; a negative request returns 1. Positive requests choose first block 2 for lengths through 2040, block 1 through 2551, otherwise block 0. The first segment capacity is 507 bytes and later capacities are 511 bytes. On a period wrap, a nonzero periodic-prefix flag reduces block 0 capacity by eight bytes. If bytes remain after six segments, return 1 with the observed mutations retained; no caller rollback behavior is modeled.

For each active block, a reservation writes a one-bit presence field and a high-bit-first nine-bit byte length. The saved payload cursor is ten bits later. An absent reservation writes only its zero presence bit. The writer preserves bits outside its output range. The first fragment is `58 38`, the full body length as a 16-bit big-endian value, and the supplied body fragment; subsequent fragments contain only supplied body bytes.

The writer accepts a `SerializedEmdfBodyFragment` typed view over serialized bytes from its caller. The wrapper does not attest provenance or act as a model-owned receipt. Fixture bodies are labeled `NATIVE_DIAGNOSTIC_BOUNDARY_INPUT_NOT_MODEL_PRODUCTION`; they are not generated or authenticated by this module. No ordinary codec audio frame is produced. Buffer bounds and local input-shape errors are API-safety outcomes, not recovered native error codes.

These carrier primitives are not wired into the production `encode` path. A future selected product caller must supply the actual model-owned body/provider receipt; diagnostic fixture bytes and lifecycle length observations are not substitutes for that receipt.

## Outer publication gate

The gate begins at progress 0, uses hop 1536 and threshold 3072, adds a hop while prior progress is below 4608, and exposes bytes only when updated progress is strictly greater than 3072. AU ordinals are allocated only to nonzero caller-visible output lengths. In the six observed calls, calls 1 and 2 publish no bytes; calls 3 through 6 publish AUs 0 through 3. Those AUs join JOC producer calls 2 through 5. Gate progress is only a warmup counter and is not an absolute ADM/source-sample clock.

## Validation and exclusions

The hash-bound handoff contains 18 controlled planner/writer boundary cases, a four-call same-owner planner sequence, six gate calls, and four native diagnostic bodies joined to published frames. Primitive writer cases use the washed deterministic byte recipe `body[i] = (37*i + 11) mod 256` and an initial `0xA5` buffer; these synthetic primitive bytes are distinct from the four native diagnostic bodies. Local tests compare 23,936 controlled output-buffer bytes and each selected natural carrier fragment. The native expected published-frame bytes are used only as an oracle for the carrier reservation/fragment bit ranges; the model output is not the complete frame.

The following remain outside this slice: protected body production/authentication, ordinary codec frame generation, ID14 parsing/semantics, whole-parent rollback after planner overflow, overlapping-buffer behavior, broader multi-fragment natural reachability, absolute ADM/host clock binding, EOF/reset behavior, and whole F13C/F13 readiness.

Wash evidence: handoff manifest SHA-256 `9c136d43219ccd8fc5623066864d27959f079c9cfe9dfcd10a5726e7fb20f049`; handoff check SHA-256 `8a0343f13fe811e5de8a64cb20ddb7fe72f7190905ef1686590240ac32cf6ff9`; 16/16 payload hashes and sizes matched; Wash replay reported zero mismatches. Independent and Wash checks both retain `whole_F13C_ready=false` and `implementation_ready=false`.
