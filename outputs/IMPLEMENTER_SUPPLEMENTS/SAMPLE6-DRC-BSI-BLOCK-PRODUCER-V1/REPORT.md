# Human Cat DRC per-block code and presence producer

## Washed behavior

The matched DEE core's selected profile curve and per-block F32 control state produce a U16 candidate code. The candidate is combined with a U16 reference value and stored in the DRC mode-0 block-code row. For the `param2 = 0` path, block 0's presence value is `(code[0] != 0)`; later blocks set presence when `code[i] != code[i-1]`. The existing DRC writer consumes those same mode-0 rows.

The profile curve comes from a typed input: I32 custom-profile flag, U8 preset selector, or four U8 custom selectors when custom mode is enabled. Built-in selector values 1..5 select rows 0..4; selector 0 selects no built-in row. The matched XML contains `film_light`, but its conversion to this U8 selector is not bound. Numeric mode 0/1 is also not identified as Line/RF.

## Matched output relevance

In AU 2843, the matched DEE stream has mode-0 `dynrnge=1, dynrng=1`; Rust has `dynrnge=1, dynrng=0`. The producer rule is compatible with this nonzero DEE block code, but the matched run's selector, F32 input, and U16 reference/candidate were not captured. This closes the static producer rule, not its input lineage or the channel-3 PCM root cause. The DRC field difference does not explain the large channel-3 residual at decoder DRC scale 0.

## Input-only vector

The vector begins after the float-to-U16 candidate has already been computed. With one block, `param2=0`, numeric profile selector `1`, branch flag `1`, reference code `1`, and candidate code `1`, the mode-0 rows become code `1` and presence `1`; the writer emits `dynrnge=1` and `dynrng=1`. Selector `1` is deliberately unlabeled and is not asserted to mean `film_light`.

No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
