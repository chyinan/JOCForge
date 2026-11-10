---
status: wash-reviewed-reference-addendum
topic: "Official-demo JOC profile index and EMDF payload configuration"
spec: "ETSI TS 103 420 V1.2.1 Tables 47-48 and 55-56"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, JOC, EMDF, profile-index]
---

# 官方演示样本 JOC profile index 补充

This addendum extends `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1`. The base package remains unchanged and supplies the full-track I0→D0, chanmap, block-two EMDF, and 1280-kbps sample facts.

## Newly washed full-track fields

- All 23,033 JOC payload-14 headers parse with `joc_dmx_config_idx=1` (U3), extension index 0, and 15 JOC header objects. ETSI TS 103 420 Table 47 maps index 1 to unshifted 7.X; Table 48 maps it to seven downmix channels.
- Each AU has one payload ID 11 and one payload ID 14 in the same EMDF group, group ID 0. OAMD reports 16 total objects: 15 dynamic and one LFE bed. E-AC-3 complexity index 16 is separate.
- I0 `dmixmod=1` is an independent Lt/Rt preference; D0 has no `dmixmod`. Both remain separate from JOC profile index, D0 chanmap `0x1A00`, OAMD object totals, and E-AC-3 complexity.

## Validation disposition

Both payload IDs 11 and 14 report `codecdatae=0` on all AUs; ETSI TS 103 420 Table 56 requires `codecdatae=1`. Other checked Table 56 fields match. Keep vendor-compatible observation and ETSI-strict failure separate; this supplement does not authorize an ETSI-strict compliance claim.

The reference sample is 1280 kbps. The target 1664 kbps = I0 640 + D0 1024 remains `NOT_EVALUATED`; profile-index evidence comes from the 1280 sample and does not prove the target encoder path. Full object position/height histories also remain unparsed.

No media or payload bytes, PCM, keys, licenses, research source code, or mode21 artifact are included. `production_allowed=false` remains in force.
