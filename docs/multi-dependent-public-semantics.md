# Public raw multi-dependent E-AC-3/JOC semantics

This document is the normative boundary for JOCForge's raw elementary-stream
multi-dependent conformance path. ETSI defines legality; JOCForge and OpenJOC
provide independent implementation evidence. FFmpeg and other players are
interoperability evidence only.

## Authorities

- [ETSI TS 102 366 V1.4.1](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf), clauses E.1.3 and E.2.8.
- [ETSI TS 103 420 V1.2.1](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf), Tables 47/48 and clauses 6.3, 8.1–8.3.

## E-AC-3 stream topology

TS 102 366 Table E.1.1 defines Type 0 as an independent substream, Type 1 as
a dependent substream, Type 2 as an independently decodable AC-3-converted
substream, and Type 3 as reserved. For this JOC path the canonical programme
starts with Type 0 independent substream `I0`, ID 0.

TS 102 366 E.1.3.1.2 defines the three-bit `substreamid` field as follows:

| Component | Legal IDs | Ordering and association |
|---|---|---|
| Independent | `I0` ID 0; additional independent IDs 1–7 | ID 0 is first; additional independent IDs are sequential in bitstream order. |
| Dependents for one independent | `D0`–`D7`, IDs 0–7 | Up to eight; immediately follow their associated independent; IDs are sequential in present order. |

Associated dependent substreams shall use the same sample rate and number of
blocks per syncframe as their independent parent. The public raw count maximum
is therefore eight dependents (`D0..D7`). This is distinct from CMAF's stricter
container rule in TS 103 420 Annex E, which allows exactly one `I0` and at most
one `D0` and requires six blocks per syncframe.

## Channel replacement and supplementation

TS 102 366 E.2.8.2 defines a single programme with more than 5.1 channels:
independent `I0` carries the compatibility downmix, while dependents carry
channels that replace or supplement it.

- With `chanmape=0`, `acmod` and `lfeon` identify the dependent channels; a
  corresponding channel replaces the matching `I0` channel.
- With `chanmape=1`, each set bit in the 16-bit `chanmap` identifies a channel
  or a public channel pair. The coded-channel order is the MSB-first order of
  set map bits. A mapped channel corresponding to `I0` replaces that channel;
  a mapped channel absent from `I0` supplements the programme.
- The same clause states that the maximum rendered channel count for one
  programme is 16.

TS 102 366 V1.4.1 Table E.1.4 maps the custom bits as: 0 L, 1 C, 2 R, 3 Ls,
4 Rs, 5 Lc/Rc, 6 Lrs/Rrs, 7 Cs, 8 Ts, 9 Lsd/Rsd, 10 Lw/Rw, 11 Vhl/Vhr,
12 Vhc, 13 Lts/Rts, 14 LFE2, and 15 LFE. The map is MSB-first in the syntax;
the hexadecimal masks used by JOCForge are consequently `0x8000` for bit 0,
`0x0200` for bit 6, and `0x0010` for bit 11.

JOCForge's canonical source-derived plan additionally requires no overlapping
dependent ownership, no dependent LFE, and no unclaimed source placeholder.
Those are deliberate fail-closed product/conformance constraints; they do not
redefine the public E-AC-3 replacement semantics.

## JOC and metadata ownership

TS 103 420 Table 47 defines the JOC downmix identities and Table 48 their input
counts:

| `joc_dmx_config_idx` | Public downmix | JOC input channels |
|---:|---|---:|
| 0 | 5.X: L, R, C, (LFE), Ls, Rs | 5 |
| 1 | 7.X: L, R, C, (LFE), Ls, Rs, Lb, Rb | 7 |
| 2 | 5.X+2: L, R, C, (LFE), Ls, Rs, Tfl, Tfr | 7 |
| 3 | 5.X with 90-degree phase shift | 5 |
| 4 | 5.X+2 with 90-degree phase shift | 7 |
| 5–7 | Reserved | — |

The Table 47 note states that LFE, when present, bypasses JOC processing. The
JOC stage therefore owns the full-band downmix channels, while the base LFE is
retained and routed separately exactly once.

TS 103 420 clause 8.1 requires decoding `I0` and all associated dependents
and passing their decoded audio to JOC. Clause 8.2 requires the EMDF container
holding OAMD and JOC side information to be in the last dependent whenever
dependents exist. Table 55 requires payload ID 11 for OAMD and 14 for JOC.
Clause 8.3 requires type-A addbsi in that same substream and sets the maximum
complexity index to 16.

The canonical ownership rule implemented and tested here is:

```text
I0                  compatibility downmix and optional base LFE
D0..D(n-1)          source-derived replacement/supplement audio, unique maps
Dn                  final dependent; sole OAMD/JOC EMDF and type-A addbsi owner
```

## Timing

TS 102 366 Table E.1.3 permits 1, 2, 3, or 6 audio blocks per syncframe.
Associated components must use the same sample rate and block count. The
canonical six-block raw JOC fixtures represent one 1,536-sample interval per
component (`6 × 256` samples), preserve component order, and require exact
sample continuity across access-unit boundaries. Short-block grouping remains
a separate capability axis and is not implicitly combined with this closure.

## Dependent-count matrix

The first column is public raw E-AC-3 legality. The second is the current
canonical no-LFE Table-47 JOC writer boundary after unique channel ownership
and source-carrier requirements are applied.

| Topology | Raw E-AC-3 legality | Canonical JOC status | Reason |
|---|---|---|---|
| `I0+D0` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Final D0 owns JOC/OAMD. |
| `I0+D0+D1` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Final D1 owns JOC/OAMD; D0 is source-derived. |
| `I0+D0+D1+D2` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Final D2 owns JOC/OAMD. |
| `I0+D0..D3` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Supported for profiles whose final Table-47 set closes. |
| `I0+D0..D4` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Canonical idx0/idx3 and 7-channel profiles as applicable. |
| `I0+D0..D5` | `PUBLIC_LEGAL` | `SUPPORTED_PASS` | Canonical seven-channel profile with five single replacements plus one pair. |
| `I0+D0..D6` | `PUBLIC_LEGAL` | `PUBLIC_LEGAL_NOT_IMPLEMENTED` | No-LFE canonical Table-47 ownership has no seventh non-overlapping dependent slot. |
| `I0+D0..D7` | `PUBLIC_LEGAL` | `PUBLIC_LEGAL_NOT_IMPLEMENTED` | Public raw count is legal; this JOCForge plan does not guess overlapping or LFE dependent ownership. |

`PUBLIC_LEGAL_NOT_IMPLEMENTED` is not an invalidity claim. It means the
combined canonical JOCForge plan is deliberately fail-closed pending an
independent profile/ownership design. CMAF rows remain invalid for `D1+`
under Annex E and are not promoted by this raw-carriage work.
