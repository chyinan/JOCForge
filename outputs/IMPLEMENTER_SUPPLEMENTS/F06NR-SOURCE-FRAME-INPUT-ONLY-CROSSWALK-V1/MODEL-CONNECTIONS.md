# Secondary producer model connections

## Source clock and owner

This package uses `source_frame s` as the source-generation name. Under the approved global transaction contract, `callback_index = s + 2` and JOC `dispatch_index = s + 1`. F06NR `frame_index` before an accepted call equals `s`; F06ND consumes the same source generation `s`. After the complete F06NR operation succeeds, the staged owner has `frame_index = s + 1`. AU number and callback index are separate clocks.

One F06NR owner contains five forward histories of 128 F32 words, five independent inverse carries of 128 F32 words, and a U64 frame index. Cold state uses positive zero for all 1,280 history words and index zero. The vectors reference each source's predecessor state instead of inserting uncomputed after-state words.

## Per-source model order

For each internal lane in `[L,C,R,Ls,Rs]` order, mapped to external rows `[0,2,1,4,5]`, and each block in `0..5` order:

1. Take the 256 samples from the already F07-delayed input row.
2. Run the selected F08 long or short forward transform using the lane's independent forward history. The flag is that same block/lane's F07 `short_consumed` value; zero is long and one is short.
3. Preserve spectrum words 0 through 216 and explicitly set words 217 through 255 to positive zero.
4. Run F06NI's inverse-256 wrapper. It applies the selected spectral split, descriptor rotations, scaling and output layout, using F06NF's fixed FFT network and parameters.
5. Apply overlap using the lane's independent 128-word inverse carry, then retain the next carry and next forward history.

This is the approved model order, not a claim about an identified native DLL callsite. All 30 block/lane operations, including block 0, advance their histories. Block 0 overlap output is discarded after its state update. Blocks 1 through 5 write 256 samples to carrier blocks 0 through 4.

## Six-row carrier

Construct a six-row, 1536-sample F32 carrier from positive zero before writing any block. Internal lanes map to external rows as follows:

| Internal lane | External row | Carrier blocks written |
|---|---:|---|
| L | 0 | 0 through 4 from inverse blocks 1 through 5 |
| C | 2 | 0 through 4 from inverse blocks 1 through 5 |
| R | 1 | 0 through 4 from inverse blocks 1 through 5 |
| Ls | 4 | 0 through 4 from inverse blocks 1 through 5 |
| Rs | 5 | 0 through 4 from inverse blocks 1 through 5 |
| LFE | 3 | none; entire row remains positive zero |

Each fullband row's final block 5 (samples 1280 through 1535) remains positive zero. F06ND reads secondary blocks 0 through 4; its auxiliary block 5 selects primary by contract.

## After-state authority

The frame-index increment and the carrier's zero slots are directly specified, so this addendum records those rules. It does not calculate the 5×128 forward histories, 5×128 inverse carries, or nonzero carrier words. Those values require numerical execution of the approved F08, F06NI, F06NF and overlap rules. The existing block-level inverse vectors and routing-only carrier hashes are not used as substitutes for that execution. These fields remain `NOT_RUN`; their exact rule sources are identified by hash in `SOURCE-IDENTITIES.json`.

The rejected cross-generation case requests source frame 1 while the owner still has frame index 0. The model contract requires `SECONDARY_INPUT_ERROR`, no normal carrier, and no state commit. This is a contract vector only; no runtime error behavior was observed.
