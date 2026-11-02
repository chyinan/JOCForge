# Sample 6 native F32-to-BSI U5 boundary

## Distinct paths

The Rust measurement path produces a `u32` containing the selected F32 word
and passes it to `dialnorm_from_selected_lufs`. The Rust conversion is F32
sentinel check, F32 subtract `0.5`, truncation toward zero, clamp to
`[-31,-1]`, then negate to U5. The full Rust run receipt currently has no
`selected_lufs_word` or getter-branch value.

Ghidra 12.2 traces a separate DEE Atmos measurement path. The aggregate
`FUN_1801769a0` takes the maximum per-source F32 at entry offset `+0x14`
(32-byte records, initial value `-120.0`), but this aggregate is not joined to
the F32 field at `EmpireEnc+0x3c`. The open-config field at `+0x58` reaches
`FUN_1800fdc60`, which checks the `-120` sentinel, subtracts `0.5` in F32,
truncates with `CVTTSS2SI`, and clamps to `[-31,-1]`. Its result is negated
only for a mismatch comparison against internal byte `+0x10a`, setting flag
`0x04`; the trace does not show this result being written to BSI.

A separate `dee_ddp_encoder.exe` DDPi API accepts integer ID `0x12`, validates
`[1,31]`, and copies the value into frame-preparation context. The native F32
path is not joined to that integer path or to a first BSI U5 write. Thus no
native U5 value or Rust/DEE scalar equivalence is established here.

## Scope disposition

The previous DEE scalar `-20.665958` and U5 `21` belong only to the bounded
`file_position 2160..2208` reference window; Rust U5 `17` is full-file. These
are not same-scope evidence. `N` neither value may be used to infer the
missing Rust selected word. Keep `selected_lufs` physical units and the
native F32-to-BSI producer join `UNBOUND`.

## Smallest useful receipt

For Rust, emit the existing `ProgrammeMeasurementReceipt.selected_lufs_word`
F32 bits together with its measurement identity. For a future same-scope
DEE comparison, capture the `EmpireEnc+0x3c` F32 word/branch at the same
input window, and trace its relation to the DDPi ID `0x12` or BSI U5 writer.
No DEE run, PCM/payload/key read, or implementation contact occurred in the
source slice.
