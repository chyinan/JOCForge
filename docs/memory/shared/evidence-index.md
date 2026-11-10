---
status: current
topic: washed-evidence-index
spec: JOCForge
confidence: package-hash-verified
last_updated: 2026-11-11
tags:
  - evidence
  - Wash
  - supplements
---

# Sanitized evidence 索引

下表路径均指向隔离仓库中的 sanitized supplements。

| Package | Manifest SHA-256 | Seal SHA-256 | 状态 | Sanitized supplement 路径 |
|---|---|---|---|---|
| `JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1` | `7876e3c53e7bda56f60c8d3a691f9082925c6ef14a7899cff21c09f8b1416714` | `2bd1c52189a97e5059f19228fef6851c6de1e7b21e743641d3709ddabd0b6c20` | Approved target contract；`production_allowed=false`；GTC、D0 algorithm、all-AU fit 未闭合 | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1` |
| `DEE-ATMOS71-I0-D0-RATE-FRMSIZ-BOUNDARY-PARTIAL-V1` | `f62ff1854f0b690501fe9dab5c61ed7b67b6f911f9cb7290b95715e2b1fa475f` | `7d1854fcfc4cbc0382a19a0ce0cd21c268a969505aa270bb4487ecbba265e8b4` | Two-frame target arithmetic；writer owner、D0 `frmsiz`/BSI 和实际输出为 `UNBOUND/NOT_RUN` | `outputs/IMPLEMENTER_SUPPLEMENTS/DEE-ATMOS71-I0-D0-RATE-FRMSIZ-BOUNDARY-PARTIAL-V1` |
| `JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1` | `0cd5b5e6a1a37961225bfd71bcbf68619baea76c48800449078919acd3499f43` | `584fdd652ce61102f182f34a133fd083e325cfcde2e9674e5cdaf2c36a576e1a` | Abstract AU target only；legacy atomicity 未证明；D0 role/algorithm `UNBOUND`；runtime `NOT_CAPTURED`；production disabled | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1` |
| `JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1` | `470fa9830a7937fcd1d5169c980f139669011e766e136ce2dc9bf7498103f787` | `aaeb22108dac10742ad410e624d9a3dae6cf9d5707b56b58697a4068d335bf81` | Numeric field-slot mapping 和 input-only vectors；semantic labels/source-role join 未绑定；native execution `NOT_RUN` | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1` |
| `DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1` | `98697c0b352bb11379d0cc942911ad34b23350a554b2432cf9e9b8405f4523a6` | `fbec0caef75f1fd2f49d6e390b090127979b189861d0482381c337d511dfa61e` | Partial static writer boundaries；I0/D0 role、AU order、source generation 和 atomic commit 未绑定；vectors `NOT_RUN` | `outputs/IMPLEMENTER_SUPPLEMENTS/DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1` |
| `JOCFORGE-ATMOS71-I0-FRAME-SYNTAX-GAP-V1` | `9f500f0785c569d67b846fa54f121b4e42c6b5b179ace98c4bdf19463c906109` | `a5ea37646195925f69cc2d53792b474a882b536cac3b26699ff4aa8679b16b3f` | Official I0 exact BSID `NOT_CAPTURED`；BSID16/strmtyp0 prefix equivalence `UNBOUND`；no prefix change authorized；production disabled | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-ATMOS71-I0-FRAME-SYNTAX-GAP-V1` |
| `JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1` | `509a4d61dbc0b20a0044a05043a99e83a7b60224aefd3df144cb394d1b30048b` | `bcce271fa5cfc7d725e4833337111c0a882a0dc09a9ebba3dfe2aeac7749b4dd` | Partial profile/header implementation authorized；`joc_dmx_config_idx=1`；full stream and official-demo acceptance `NOT_RUN` | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1` |
| `JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1` | `a212684b565f93ca23ce5dbab458d6ee74cfb8e231b0ce7210db1da4e098aefa` | `abb4f166ed720bd0c0a90e393e0b74216fe40b72ee77d1d813446371a82cbe6b` | Optional `--joc-layout flat7x` routes immutable profile configuration to the JOC header；real output waits for Wash timeline continue receipt | `outputs/IMPLEMENTER_SUPPLEMENTS/JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1` |
| `ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF-V1` | `530867a0e15a82dcb461d540e2d07ef4e53de5fdd66131109b5c0110a7d3b884` | `7ca3956885e570fd5d51aadeea1b74b94e522380c487fd13ee86cdc71e5dcf5a` | Hash-checked target handoff；typed I0/D0 plan, CRC-checked byte sequence, F15 owner/ledger, and isolated staging-file receipt adapter are locally materialized；`production_allowed=false`；GTC/product binding, syntax, runtime owner join, D0 audio/writer remain open | `outputs/IMPLEMENTER_SUPPLEMENTS/ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF.md` |
| `FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` | `18de94b6b18a13222df8e9eae018f77c49fa735007869b99255b97a1cb5ae0d1` | `8879c0417bf60db4d8dc9dec1fbbeccd884978af6460f304cfb8034052e4d3ee` | Wash-reviewed staged-state rules：同AU source-generation标签相等；PublishedAuIndex/owner generation独立；fatal failure retire attempt；完整计数后允许模型 finalize proposal；`production_allowed=false`；callback join、GTC、final exposure、audio/BSI/EMDF joins remain open | `outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` |
| `FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1` | `1034ef415113b666acc5aefaa3207230040f05d9ebc05a33933bffc50b4ff088` | `16d925352e9b197e016b2554f9c2ae609e8f8890e3dca81c4eb591f737b41210` | Synthetic F13/I0/D0 receipt-gate pair and listed rejection cases replayed through F15 owner validation；fixture bypasses F13 parser/F12 owner stages；valid parser container, runtime join and product acceptance remain open；`production_allowed=false` | `outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1` |
| `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` | `f7f8630c3f18e990324f013e70486c561de6816a844f852f18f03a82ce28fbfc` | `2d883e278bb83fa0d617610f94b8a87fa5849ac7bf19bdf08818d8d30d50740e` | Hash-bound reference summary: I0 legacy AC-3 BSID6 2,560 B then D0 dependent E-AC-3 BSID16 2,560 B, 1,280 kbps sample; target 1,664 kbps / 4,096 B D0 remains separate; profile index and target output gates remain open; `production_allowed=false` | `outputs/IMPLEMENTER_SUPPLEMENTS/OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` |
| `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` | `c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a` | `0fb4c7f8a7a49488a97586b7e3eb6febc0542cf0044569fe37e65dc07d85757d` | V1 addendum: all 23,033 payload-14 headers report `joc_dmx_config_idx=1`/extension0/15 objects; IDs 11/14 have `codecdatae=0` while ETSI Table 56 expects 1, so vendor-compatible observation and strict failure stay distinct; 1,664-kbps target remains `NOT_EVALUATED`; `production_allowed=false` | `outputs/IMPLEMENTER_SUPPLEMENTS/OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` |
