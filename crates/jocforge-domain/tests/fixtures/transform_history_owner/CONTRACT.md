# Transform history owner and continuity

## Scope

This contract covers the selected format-7 history bank layout, its observed cold constructor, six source-slot ordering, and bounded continuity across native calls. It also covers one local workspace initializer. It does not define a full frame/ADM clock, owner reset after fatal/EOF, input padding/drain, or sample-position mapping.

## History layout

- There are six native source slots in the selected construction.
- Each slot owns two distinct banks: primary and companion.
- Each bank contains 128 F32 words (512 bytes); one logical slot is 1,024 bytes.
- The native constructor clears the region covering all 12 banks. The active constructor returned 0 and the 12 initial-state digests were zero.
- A clean Rust owner may use a typed structure; reproducing the native object ABI or raw pointer layout is not required.

For selected format 7, source slots 0..4 are full-band and slot 5 is the LFE consumer position at this boundary. Mapping external ADM/WAVE speaker lanes to these slots is a separate upstream contract.

## State transition

The owner stores history only; the existing transform fold consumes and updates it. For each 256-word input block, process `i=0..63` and the two tuples in order:

```text
(a,b,o,s,h) = (2i,   255-2i, 128+2i, 129+2i, 64+i)
(a,b,o,s,h) = (2i+1, 254-2i, 127-2i, 126-2i, 63-i)
```

Let each operation round to F32 separately. Compute `A=input[a]*window[b]`, `B=input[b]*window[a]`, `C=input[b]*window[b]`, `D=window[a]*input[a]`; read `old_history[h]` before writing it. Primary writes `temp[o]=R(A+B)`, `temp[s]=old_history[h]`, `history[h]=R(C-D)`. Companion writes `temp[o]=old_history[h]`, `temp[s]=R(A-B)`, `history[h]=R(C+D)`. Exact window bits are bound in `PARAMETERS.json`.

## Continuity evidence and bounds

Across six observed outer generations, each of the six source slots and both banks receives six block calls per generation: 432 fold calls total. The first call for each of 12 source/bank slots begins with zero history. All following 420 transitions use the same owner and state slot, with each before digest equal to the preceding after digest. The controlled zero/impulse/periodic recipes independently reproduce 1,536 history words.

An outer generation is an observed encoder process-call domain. It is not identified as an EC3 AU or an absolute ADM sample position. The evidence binds owner and bank continuity only.

## Workspace initializer boundary

For a controlled descriptor of 1,320 bytes and arena of 65,536 bytes, each prefilled with `0xA5`, the initializer returns 0 and sets:

- 30 POWER rows to zero, each with 253 U32 words;
- five bias rows to zero, each with six I16 values;
- 30 scratch-exponent rows to zero, each with 253 I16 words;
- six LFE scratch rows to zero, each with seven I16 words;
- one shared 253-word I16 work row to zero;
- 30 full-band current-row pointers and six LFE current-row pointers to null/unbound.

The final 16 arena bytes and 24 descriptor bytes remain `0xA5`. Undefined descriptor bytes are not claimed initialized. Scratch exponent rows are not bound current rows. This initializer is not a full codec reset and has no established invalid-buffer error behavior.

## Remaining lifecycle

No explicit same-owner reset, fatal recovery, EOF partial-block padding/drain, history retirement, or absolute ADM sample clock was observed. The next evidence methods are to trace the actual reset/finalization callers, compare a restart on a fresh owner, and run a normal nonmultiple-of-256 end-boundary job. Keep these boundaries open until measured.
