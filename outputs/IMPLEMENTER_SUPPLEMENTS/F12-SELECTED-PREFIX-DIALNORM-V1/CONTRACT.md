# F12 selected prefix and programme dialnorm

This supplement separates the closed selected-profile wire fields from the still-unbound cross-stage identity receipt.

## Fixed prefix

For the selected 48 kHz, 1024 kbps, 5.1 (`acmod=7`, `lfeon=1`), six-block, one-independent-substream, 4096-byte frame profile, F12S.7 writes these unconditional fields MSB first:

| Field | Width | Value | Inclusive frame-relative bits |
|---|---:|---:|---:|
| `syncword` | 16 | `0x0B77` | 0–15 |
| `strmtyp` | 2 | 0 | 16–17 |
| `substreamid` | 3 | 0 | 18–20 |
| `frmsiz` | 11 | 2047 (`0x7FF`) | 21–31 |
| `fscod` | 2 | 0 | 32–33 |
| `numblkscod` | 2 | 3 | 34–35 |
| `acmod` | 3 | 7 | 36–38 |
| `lfeon` | 1 | 1 | 39 |
| `bsid` | 5 | 16 | 40–44 |

The prefix is exactly 45 bits:

`000010110111011100000111111111110011111110000`

`0b7707ff3f80` is only a display byte string with three trailing zero pad bits; those three bits are not part of the prefix or the next field.

## Dialnorm value

The next field is the five-bit `dialnorm` at bits 45–49. It is derived from the finalized full-programme `selected_lufs` F32; it is not a fixed profile constant and has no default:

1. If `abs(R32(selected_lufs - (-120.0))) < 0.000001`, set `dialnorm = 31`.
2. Otherwise compute `t = trunc_toward_zero(R32(selected_lufs - 0.5))`, clamp `t` to `[-31, -1]`, then set `dialnorm = -clamped_t`.

F06ND consumes the same programme dialnorm for each source generation. Its selected scalar mappings include `n = R32(S16(-256*d) * 2^-15) = -d/128` and `dialnorm_offset = R32((31-d)/144)`. The later `compre` bit follows the five-bit dialnorm at bit 50; the existing F06ND-to-F12 BSI supplement defines the per-source RF/line projections.

## Identity status

The clean model requires one finalized programme measurement to feed every main-source BSI/ID1 projection and rejects rolling-prefix measurements. Its input-only sequence demonstrates that reuse inside one model state instance. The existing measurement digest does not identify a unique programme/input, F06ND accepts a scalar dialnorm, and the F12 BSI input contract does not name an identity-bearing receipt. Therefore the cross-stage typed programme/source join remains `NOT_READY`.

`MINIMUM-RECEIPT.json` records evidence fields for closing that join. It is an evidence request, not a product API declaration or policy. The fixed prefix and numeric dialnorm formula are `STATIC_CLOSED`; the whole F06ND → finalized measurement → F12 identity join is not.
