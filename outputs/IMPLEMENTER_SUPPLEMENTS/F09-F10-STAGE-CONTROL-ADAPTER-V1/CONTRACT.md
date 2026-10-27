# F09/F10 stage-control crosswalk

## Decision

The four implementation questions are covered by existing approved F07/F09/F10 clauses and frozen comparison material. No new numerical algorithm or native reverse engineering is required.

The remaining work is API naming and adapter shape. In particular, `refresh_marker_before_i16` is a per-block input derived from F07 structure flags, not persistent policy state; `normalization_span_u8` belongs to an F10 anchor request after strategy selection, not to the pre-refresh control object.

## Channel and clock domains

FBW lanes use internal order `[L,C,R,Ls,Rs]`, lane indices `0..4`. External audio `[L,R,C,LFE,Ls,Rs]` maps through `[0,2,1,4,5,3]`. LFE is external channel 3 and has a separate exponent path with no FBW short/other/structure input.

Each source has six blocks `0..5`. FBW current/work rows have 217 I16 entries; LFE rows have 7. F09 creates zero work rows at source-frame start. Work survives only across that source's six block calls and is discarded before the next source.

## F07 structure flag to F09 marker

F07 produces `short_consumed[6][5]`, `other_consumed[6][5]`, and `structure_refresh[6][5]` in `[block][lane]` order.

For every FBW lane:

- `structure_refresh[0][lane] = 1`;
- for block `b=1..5`, the value is the OR of current `other_consumed`, a change in current versus previous `short_consumed`, a change in SPX, and a change in end-bin;
- selected SPX is always 0 and selected end-bin is always 217, leaving current other or short transition as the varying terms.

The F09 refresh call for `(block,lane)` receives:

```text
refresh_marker_before_i16 = exact_i16(structure_refresh[block][lane])
current_i16[217]          = POWER raw exponent row for this block/lane
work_before_i16[217]      = zero row at block0, prior work_after for blocks1..5
refresh_mode_i16          = 1
refresh_special_i16       = 0
active range              = [0,128)
```

`refresh_marker_before_i16` must not be taken from `short_consumed`, and the prior block's returned marker is not carried as the next block's marker-before. The state carried between blocks is `work_after_i16[217]`. The refresh leaf may upgrade a zero marker to one after its difference test; its returned `marker_after_i16` is the refresh decision consumed by strategy selection.

Authority: `specs/07-history-and-transient.md:50-58,82-92`; `specs/09-exponents.md:9,23-33,61-63`.

## Meaning of `refresh_special_i16`

The approved clean contract names the active branch `mode`: selected ordinary FBW uses mode 1; LFE uses mode 0. The frozen leaf has `mode_a1` and `mode_a2`; comparison calls use `mode_a1=mode` and always `mode_a2=0`.

Therefore, if `FullBandExponentStageControls.refresh_special_i16` denotes frozen `mode_a2`, its selected value is exact I16 zero. It is a selected-profile constant, not a source-state producer.

The adapter must also supply or internally fix the active mode as FBW `1`. If implementation currently uses `refresh_special_i16` as its only mode field, the name is ambiguous: setting it to zero as the active mode would incorrectly select LFE-style refresh. Resolve this locally by documenting two semantics or by keeping active mode internal and naming the exposed field as the frozen special/mode-a2 value.

Authority: `specs/09-exponents.md:9,27-33`; frozen comparison `verify_authorized_policy_delivery.py:109-120`; frozen policy identity record fields `refresh_mode_a1=1`, `refresh_mode_a2=0`, `backward_mode_a1=0`.

## Marker-after to strategy to F10 span

For each FBW lane, collect the six F09 `marker_after` values. F09.4 scans blocks from 5 to 0 and produces strategy:

- non-refresh block → strategy 0;
- refresh distance `<2` → strategy 3;
- distance `2..3` → strategy 2;
- distance `>=4` → strategy 1;
- after that selection, `short_consumed=1` overrides the block strategy to 3.

F10 span is derived only for nonzero strategies:

| strategy | F10 action | `normalization_span_u8` |
|---|---|---|
| 0 | reuse the nearest prior anchor's complete normalized plan and gain | no normalization call; no span value |
| 1 | normalize anchor | 1 |
| 2 | normalize anchor | 2 |
| 3 | normalize anchor | 4 |

Do not encode reuse as span 0: F10 accepts spans only `1,2,4`. If the local API requires a nonoptional `normalization_span_u8` on every block, its shape is wrong. Use an optional anchor request or call F10 only for nonzero-strategy blocks.

Authority: `specs/09-exponents.md:35-41`; `specs/10-exponent-normalization.md:7-15,45-55`.

## Raw rows used by F10

F09 has two exponent domains:

- POWER raw/current/work rows drive refresh and strategy;
- PRIMARY raw rows come from strict-clipped primary coefficients and drive serialized/decoded exponents.

F10 consumes PRIMARY raw rows, never POWER `current` or `work_before`. For each FBW lane it receives six rows of 217 I16 values in `0..24`, the six final strategies, and six U2 gain ranges. It merges every reuse interval by taking the per-bin minimum and minimum gain, normalizes each nonzero-strategy anchor with span `1/2/4`, and gives every reuse block the anchor's full expanded 217-entry exponent row and inherited gain.

The F12 BAP/mantissa consumer therefore receives `decoded_exponents_for_BAP[6][5][217]` from the F10 plan, not the F09 refresh work rows.

Authority: `specs/09-exponents.md:13-17,51,61-65`; `specs/10-exponent-normalization.md:7-15,45-55`; `specs/12-selected-source-frame-composition.md:18,34-46`.

## Independent LFE path

LFE does not consume F07 short, other, or structure flags.

Per source:

- create a 7-entry zero work row;
- create LFE `marker_before` sequence `[1,0,0,0,0,0]`: block0 is the mandatory structure refresh; later blocks have no forced external structure marker;
- call F09 refresh in active mode 0 with special/mode-a2 0, range `[0,7)`;
- carry only `work_after[7]` to the next block;
- use each returned `marker_after` directly as LFE strategy `1/0`.

The returned marker can turn a later block from zero to one based on the LFE mode-0 difference test. The fixed marker-before sequence is not the final refresh sequence.

LFE PRIMARY raw input is a six-row `6x7` I16 plan derived from the strict-clipped LFE MAIN coefficient rows. For each refresh interval, take the per-bin minimum across the interval. Every anchor normalizes with span 1, option 1, and selected independent LFE gate 0; every reuse block receives the anchor's full expanded 7-entry row. LFE has no gain-range field.

Authority: `specs/09-exponents.md:23,27,33,41,61-63`; `specs/09-lfe-gate-mantissa-boundaries.md:27-46`; `specs/10-exponent-normalization.md:47,51-55`.

## Minimal adapter mapping

```text
FBW source inputs:
  structure_refresh_i1[6][5] <- F07 structure output
  short_consumed_i1[6][5]    <- F07 short output
  power_raw_i16[6][5][217]   <- F09 POWER exponent producer
  primary_raw_i16[6][5][217] <- F09 strict-clipped PRIMARY producer
  gain_range_u2[6][5]        <- F09G

For each FBW lane:
  work = [0;217]
  for block 0..5:
    marker_before = structure_refresh[block][lane]
    refresh(current=power_raw[block][lane], work, marker_before,
            active_mode=1, special_mode=0, begin=0, end=128)
    work = work_after
    refresh_decision[block] = marker_after
  strategy = reverse_strategy(refresh_decision, short_consumed[:,lane])
  plan = normalize_PRIMARY_rows(primary_raw[:,lane], strategy,
                                gain_range[:,lane], spans 1/2/4)

LFE source inputs:
  power_raw_i16[6][7]
  primary_raw_i16[6][7]
  independent gate_i16 = 0

  work = [0;7]
  marker_before = [1,0,0,0,0,0]
  refresh mode=0; marker_after becomes strategy
  normalize PRIMARY reuse intervals with span1, option1, gate0
```

This is an interface crosswalk over existing algorithms. It defines no new exponent math or persistent state.

## Limitations

No Rust implementation source, PCM, DLL execution, keys, frame body, or native state was read. Frozen source texts for the three model anchors are currently unavailable; their identities are manifest-bound and the operative rules above are authorized by the approved clean specs and comparison core. The local API names are not a native ABI.
