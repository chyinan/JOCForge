# Human Cat DRC profile table initialization

## Washed behavior

The matched DEE core initializes two DRC modes, each with six block entries. Each mode has a U16 presence row and a U16 code row; all entries start at zero. The mode rows are distinct from the profile-curve table selected by the core's typed profile inputs.

The profile inputs are an I32 custom-profile flag, a U8 preset selector, and an optional record of four U8 custom selectors. With custom mode disabled, preset selectors 1 through 5 select built-in curve rows 0 through 4; selector 0 selects no built-in row. With custom mode enabled, the four U8 fields select their rows independently. These are raw selector semantics only: the matched XML strings `film_light` do not yet map to a particular U8 value, and the two DRC modes are not yet identified as Line versus RF.

## Matched-sample consequence

An earlier washed syncframe comparison found a first-block DRC difference at AU 2843: the presence flag is set in both streams, while DEE emits `dynrng=1` and Rust emits `dynrng=0`. This new package binds the table's zero initialization and profile selector contract, but does not find the later store that changes the block entry from zero. The positive per-block producer remains open. This DRC mismatch does not explain the large channel-index-3 PCM residual measured with decoder DRC disabled; its contribution when decoder DRC is enabled is not isolated.

## Input-only vector and next work

The input-only vector uses custom flag `0`, raw preset selector `1`, null custom record, mode `0`, and block `0`. Starting from initialized zero rows, it emits `dynrnge=0` and omits the `dynrng` U8. Selector `1` is deliberately unlabeled; it is not asserted to mean `film_light`.

Continue from this state to locate the positive per-block presence/code producer and the XML-string-to-U8 selector mapping. Do not infer either from enum order or the input-only selector. No encoder run or media capture was performed for this package.

No PCM, encoded payload, private key, license material, native code, or disassembly is included.
