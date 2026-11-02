# DEE E-AC-3 fullband U4/U7 exponent writer boundary

## Scope

This supplement covers the DEE native E-AC-3 fullband exponent syntax writer in `dee_ddp_encoder.exe`. It is separate from the Atmos DDP JOC coefficient payload path and from the outer six-block DDP frameset stuffer.

Status: `STATIC_WRITER_CONTRACT_READY_FOR_IMPLEMENTATION; REAL_FRAME_GROUP_COUNT_AND_ADM_LANE_JOIN_UNBOUND`.

## Best-supported writer rule

For each block and internal fullband lane, read the strategy from `state + 0x1bae8 + 0x3a + 10 * block + 2 * lane`. Strategy `0` omits this exponent payload. For nonzero strategy, read lane code pointer `*(u64 *)(state + 0x143b8 + 8*j)`, where `j = 5 * block + lane`.

1. Write one U4 value from `u16(lane_code_pointer + 0)` using the shared cursor.
2. Read the U7 group count `N = u16(state + 0x141c8 + 2*j)`.
3. Write `N` U7 values from the adjacent code array beginning at `lane_code_pointer + 2`, using the same cursor.

The exponent section advances by `4 + 7*N` bits. The same `FUN_140362640` bit writer is called first with count 1/width 4 and then with count `N`/width 7; Ghidra confirms the cursor register is unchanged between calls. The model input-only example `217 bins -> 216 differences -> 72 groups` is a cursor example only, not a DEE runtime count or an ADM-lane mapping.

The per-block strategy selector returns values 0 through 3. The 1024 kbps / 5.1 profile's actual per-block selector and the real ADM frame's `N` remain unobserved; preserve them as per-block/per-lane inputs rather than deriving them directly from bitrate or channel count.

## Confidence and limits

Confidence is high (`0.97`) for the native field order, widths, count field, and shared cursor. The exact numeric group count for a real ADM frame, producer of the U4/U7 code values, and physical ADM row to internal lane mapping are not established by this static package. No DEE run or audio-payload read was performed; the seven authorized ADM files in a separate metadata preflight supplied zero exponent-code observations.

This is an actionable E-AC-3 core syntax contract. It may help address malformed exponent syntax, but it does not establish that it is the sole cause of the observed AU0 `exponent -1 is out-of-range` error. Apply it to the current encoder's matching lane/block writer, run the user-authorized `人是猫` input, and check the first AU with a parser. Keep the JOC payload changes separate.

## Input-only vectors and exclusions

`INPUT-ONLY-VECTORS.json` checks cursor intervals for an inactive strategy, zero U7 groups, one U7 group, and a 217-bin model row. It includes no native output bytes and is not a captured DEE lane.

No DEE executable, disassembly, decompilation, ADM audio, PCM, private key, or license material is included. Function-body identities and image hash are preserved as hashes only.
