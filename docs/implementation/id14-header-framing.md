# ID14 header/framing washed input

Classification uses the research evidence classes: `NATIVE_OBSERVED`, `RUNTIME_CONFIRMED`, `STATIC_ONLY`, `MODEL_POSTCOMPARE`, `NOT_OBSERVED_IN_BOUNDED_COVERAGE`, and `PRIVATE_OR_NON_DELIVERABLE`.

## Fixed fields

The writer consumes unsigned integer control values and writes them MSB first in this order:

| Field | Wire bits | Selected native input | Native validation/status | Evidence |
|---|---:|---:|---|---|
| downmix index | 3 | 3 | 0..4: status remains 1; 5..7: encodes and status becomes 0; >=8: return code 3 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| object count minus one | 6 | 15 -> 14 | writer encodes count 1..64; 1..15: status remains 1; 16..64: status becomes 0; 0 or >=65: return code 3 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| extension config | 3 | 0 | 0: status remains 1 and omits marker; 1..7: status becomes 0 and appends marker; >=8: return code 3 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| clip gain index | 8 | 128 | 0..255: status remains 1; >=256: return code 3 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| sequence count | 10 | 0, then1 | 0..1023: status remains 1; >=1024: return code 3 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |

The fixed header occupies 30 bits. It is followed by one descriptor per object; all fields continue in the same bitstream without byte alignment. The descriptor list therefore makes the header length variable.

## Per-object descriptor input record

The native producer supplies 28-byte records containing seven U32 fields. These offsets describe the observed ABI record, but clean implementation may represent the fields semantically and must not serialize this native struct as such.

| Field order | Native record offset | Wire rule | Selected live value | Evidence |
|---|---:|---|---:|---|
| present | 0 | 1 bit; nonzero native value enters following fields, low bit is written | 1 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| kind code | 4 | 3 bits; 0..7 | 5 | NATIVE_OBSERVED + RUNTIME_CONFIRMED; semantic enum mapping STATIC_ONLY |
| mode code | 8 | 2 bits; 0..3 | 1 | NATIVE_OBSERVED + RUNTIME_CONFIRMED; semantic enum mapping STATIC_ONLY |
| nested mode | 12 | 1 bit; 0 or1 | 0 in live writer1/3, 1 in writer2 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| nested count | 16 | 1 bit encoding count-1; count must1 or2 | 1 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| selected index0 | 20 | 5 bits if nested mode1; 0..31 | 23 or14 | NATIVE_OBSERVED + RUNTIME_CONFIRMED |
| selected index1 | 24 | additional 5 bits only if nested mode1 and count2; 0..31 | 0 in observed live records | NATIVE_OBSERVED + RUNTIME_CONFIRMED |

When a selected index is outside the channel limit, or a two-index pair is not strictly ascending, native encoding continues but clears the status word. The writer accepts `present=2`, writes its low bit as zero, and still enters the field branch because the full value is nonzero; the selected producer emits canonical 0/1, so do not infer support for other values.

## Body, trailer, padding and capacity

The body bit sequence is appended immediately after all descriptors. The input is a byte buffer plus a valid-bit count. If extension config is nonzero, append `0xD0E1` as 16 MSB-first bits after the body. Finally append zero bits to the next byte boundary. Padding is excluded from valid-bit count and included in output length.

The body builder in the selected parent stages a 4,344-bit attempt buffer. The native bit writer returns code2 when a field does not fit; it can leave already-written bits in the attempt buffer. The output length remains unchanged on any failure. Native field/descriptor validation returns code3; null writer/descriptor input returns code1. Status is initialized to1 and can be cleared to0 for unsupported profile/descriptor conditions while the writer still returns success and emits bytes.

## Validation and failure precedence

The native writer validates in this order: writer/control/output/status pointers; downmix range; object-count range; extension-config range; descriptor-list pointer; clip-gain range; sequence range; then each object in array order (`present` bit, kind range, mode range, nested mode, nested count, selected indices). A valid field write that exceeds remaining capacity returns code2 at that point. A field range error returns code3 before that field is written. This order was probed with multiple simultaneous invalid inputs; see the `PRECEDENCE_*` vectors.

Failures can leave earlier bits in the supplied output buffer, but the output-length in/out value remains its original capacity. The observed parent rebuilds the body and retries after either body-builder or ID14-writer failure; selected profile and sequence fields update after the successful retry-loop exit. Only those named post-success fields are established here; rollback of every other parent field remains unproven.

The separate status word is not the writer return code. The writer can return success while status is zero for an unsupported downmix, extension value, object count above15, channel index outside the selected channel limit, or a two-index pair that is not strictly ascending. In the selected parent call, retry is keyed to the writer/body return code; no status-word rejection branch was observed in that bounded call path.

## Parent retry and commit

The parent assembles body bits into attempt-local scratch, then invokes the writer. A body or writer error enters the representation retry path and rebuild loop. The writer updates the output length only on success. The parent updates the selected profile/downmix state and sequence after a successful writer exit; sequence wraps1023 to1. Static review does not prove rollback of every other JOC state field.

## Evidence vectors and open semantics

Address-free exact vectors, success bytes, bit lengths, padding and SHA-256 are in `ID14-HEADER-FRAMING-VECTORS.json`. They include a header-only case, a header plus five body bits, a non-byte-aligned header/body splice, a selected 15-object/900-bit capacity boundary, and failed precedence cases.

The `OBJECT_COUNT_64_STATUS_ONLY` recipe uses `descriptor_template_repeat_count=64`; the named `descriptor_template` is expanded exactly 64 times in object order. The four selected 15-object recipes (`HEADER_15_LIVE_SHAPE`, `HEADER_15_LIVE_SHAPE_PLUS_900_ZERO_BODY_BITS`, `LIVE_SHAPE_BODY_900_CAPACITY_EXACT`, and `LIVE_SHAPE_BODY_900_CAPACITY_ONE_SHORT`) likewise use `descriptor_template_repeat_count=15`. These are explicit input recipes, not counts inferred from expected bytes.

Failure recipes bind the call-before buffer explicitly with `initial_output_buffer_fill.byte_hex="aa"` and `initial_output_buffer_fill.length_bytes=capacity_bytes`; `initial_output_buffer_hex` is the complete pre-call sentinel buffer. `native_prefix_hex` is the complete post-call buffer, including any partial writes followed by untouched sentinel bytes. The output-length assertion remains separate: failures leave the original length value unchanged. The `0xaa` tail is therefore a test sentinel, not an additional writer output contract.

`IMPLEMENTATION_READY=NO`: kind/mode enum meanings and their complete upstream producer for all profiles remain STATIC_ONLY; retry rollback beyond named post-success state remains NOT_OBSERVED_IN_BOUNDED_COVERAGE. No model postcompare was run.

Private authentication is excluded. No raw PCM, history or ADM source metadata is included.
