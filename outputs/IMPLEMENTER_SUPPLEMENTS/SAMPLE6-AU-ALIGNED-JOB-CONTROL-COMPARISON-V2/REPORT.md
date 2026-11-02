# Sample 6 DEE job controls vs the Rust full-run template

## Controls now directly comparable

The DEE job XML hash is `edd0e5b173d763e63db5405ea85aa9783bbd0477060233726843ec6f1f2eda2f`.
The Rust full run used source commit `216326f...`; the bound
`encode_request.rs` source constructs its selected full-file job XML with
the same explicit audio controls as the DEE reference:

- 1,024 kbps data rate (`1024` kbps vs `1,024,000` bit/s);
- DRC line and RF profiles `none`;
- preferred downmix `LoRo`, all four LoRo/LtRt center/surround levels `-3`;
- surround/height trims `auto`.

The XML route is `encode_to_atmos_ddp`; Rust sets `joc=true`, but the XML has
no explicit JOC boolean. The DEE reference uses a 4-second `file_position`
window while the Rust template leaves start/end blank for full-file output.
This range difference is expected for the bounded comparison.

Thus LoRo/−3 and line/RF DRC profile mismatch are no longer the leading
explanation for the measured 5.1 core PCM difference: these controls match
at the DEE XML and Rust source-template level. The actual Rust runtime XML
digest was not captured, so equality is not an exact run receipt join.

The Rust target profile reports 48 kHz, 5.1, `acmod=7`, LFE on, and six
blocks per access unit. The DEE XML does not explicitly state those fields;
the bounded M4A probe reports 48 kHz and `5.1(side)`, but this does not by
itself bind the omitted XML fields or frame-level values. Channel map,
measurement word, selected ID-0 branch, and JOC spatial rendering remain
separate unresolved audio controls.

## Next audio investigation

With rate, LoRo/−3, DRC `none`, and trims aligned in source configuration,
prioritize same-window input row ordering, selected JOC profile selector,
measurement/gain state, and the per-block coefficient transform. Do not
attribute the 5.1 core residual to one field without a discriminating
measurement. The current 4-second PCM result is still a bounded core
mismatch, not a full-song or Atmos spatial verdict.

## Scope

This supplement compares whitelisted job XML and Rust template source fields
only. No job XML, path, key, license, media, PCM, or encoded payload is
included. Output values were not used to infer missing settings.
