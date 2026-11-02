# Line DRC profile string and first integer consumer

## Wash disposition

The upstream R5 package update was independently rechecked against its manifest, SHA256SUMS, package seal, and six payload hashes. It extends the earlier plugin-local registry trace through the Line profile descriptor's string storage and first integer-valued consumer.

## Bounded value path

- The Line descriptor is at owner `+0x6d0` and is registered under `drc:line_mode_drc_profile`.
- The descriptor setter validates the incoming string against the descriptor's enum strings. It stores the validated string and marks the descriptor set; it does not convert to an integer in the setter.
- Initialization calls the same setter with default string `film_light`.
- The first direct consumer reads the stored string and searches a profile tree. On a match, it copies the matched node's U32 to `param_2+0xa0`.

## Audio sink boundary

The exact `film_light` tree-node U32 is not enumerated. A prior static package locates a Line DDP_COMPR U32 at `param3+0x28` in `FUN_1801aa730`, but this package does not join the selected tree value to the downstream encoder input. The string-to-code value and code-to-encoder sink remain `UNBOUND`; no DRC DSP/output behavior was run or established.

This is enough to model a validated string default and a tree-lookup boundary, but not enough to implement the numeric DRC profile behavior or claim audible parity. No DEE run or PCM/key/license access occurred.
