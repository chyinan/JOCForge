# Sample 6 selected-LUFS to DialNorm producer boundary

## Rust measurement path

The reviewed Rust path maps ADM CHNA `track_index_1based-1` to an interleaved
BWF row. `CompleteSampleBlock` F32-word lanes feed energy records containing
mantissa words and exponents; integrated and ungated meter states feed
`compute_programme_measurement_values`. `di_off_getter` selects the ITU
internal value when it is above `LOUDNESS_FLOOR_WORD`, otherwise the ungated
internal value, then forms `selected_host_word` using the F32 scale
`128.0f` (`0x43000000`). This numeric scale does not establish a physical
unit; `selected_lufs` physical units remain `UNBOUND`. The run receipt does
not record which getter branch or selected word was used.

`dialnorm_from_selected_lufs` consumes the selected F32 bit word. The current
Rust conversion checks the `-120.0f` sentinel, subtracts `0.5f` in F32,
truncates toward zero to I32, clamps to `[-31,-1]`, negates, then emits U5
`[1,31]`. No F64 arithmetic is used in this conversion path. These are Rust
implementation facts; this supplement does not claim they are independently
proven native DEE behavior.

## Run-scope comparison

The keyless full Rust run on the authorized input recorded U5 `17`, but its
receipt omitted the selected F32 word. A same-input DEE receipt records
scalar `-20.665958` and U5 `21` only for `file_position` 2160..2208. Under a
`selected_lufs` interpretation that scalar predicts `21`; it is not the
Rust scalar, and the bounded DEE window is not same-scope evidence against
the full-file Rust value. Do not infer a formula mismatch from `17` versus
`21`.

`ProgrammeMeasurementReceipt` already contains a non-sensitive
`selected_lufs_word` field. Surfacing that F32 bit word in the run receipt,
bound to the existing programme/attempt/input/config/meter-terminal
identities, would make the actual Rust U5 reproducible without retaining PCM
or payload.

## Limits

The selected word's physical unit, selected getter branch for the run, exact
native DEE input scalar, and same-scope full-file comparison remain open. No
new DEE run, PCM/payload access, private-key access, or old comparison-group
use occurred in this audit.
