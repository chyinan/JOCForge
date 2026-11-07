# DEE line-mode DRC package seal-chain refresh

## Disposition

This is a provenance-only Wash addendum for three existing DRC line-mode packages. The parent seal was regenerated, so the mode-1 and mode-2 child manifests/seals changed to bind the current parent. Wash recomputed every current payload hash. The investigation findings and `UNBOUND/NO` boundaries are unchanged; do not treat this as new audio-effect evidence.

## Current package identities

| Slice | Current manifest SHA-256 | Current seal SHA-256 | Current artifacts | Boundary |
|---|---|---|---:|---|
| Default encode / codec consumer | `c1ccfd222e409453bf152dc43d3c2c146ac0967d017038dcc3e395a0df1fcfec` | `7ccc4c414a307627b8717bcd15a39b43da98e3733f4cb2cca8a340ad5167d701` | 6/6 payloads | `+0xa0` maps conditionally to frame record; no typed transfer to DDPJocEnc `+0xd8`, first method, or writer; final effect `UNBOUND/NO`. |
| Mode 1 | `98355e72c205b32e9b47ec2f37e628f369aaedb7b3de8a1681ab584abc626143` | `415f25a4658ba590506ff10760ec8b6b158c8b3901b7f76b8bce5f35e69d8308` | 6/6 payloads | First item consumer receives record but reads `+0x9c`/vectors, not `+0xa0`; effect `UNBOUND/NO`; runtime `NOT_RUN`. |
| Mode 2 | `582bb849c498acd32a06fbf193bb0553141a416226e30ec1a8c01149d35f46ab` | `54291d980e4289e48a604d4c3a08c6617b0a4c644b9fb7fa80325f1d28317d9a` | 6/6 payloads | First item consumer receives record but reads `+0x9c`/vectors, not `+0xa0`; effect `UNBOUND/NO`; runtime `NOT_RUN`. |

The regenerated main seal is the current parent seal referenced by the two refreshed child source identities. The prior hash-bound Wash boundary reports remain valid for scope; this addendum replaces only the source seal-chain references. The normal and `python -O` checker results are agent-reported; Wash verified hashes and did not rerun the checkers.

## Source packages

- `adm_bwf_ddp_drc_line_mode_codec_consumer_boundary_wash_v1`
- `adm_bwf_ddp_drc_line_mode_alternate_encode_consumer_boundary_wash_v1`
- `adm_bwf_ddp_drc_line_mode_mode2_encode_consumer_boundary_wash_v1`

No DEE run, media/payload/key/license read, or implementation-source modification occurred.
