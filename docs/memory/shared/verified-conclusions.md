---
status: verified
topic: washed-7.1-flat7x-conclusions
spec: JOCForge
confidence: sealed-package-backed
last_updated: 2026-11-11
tags:
  - Wash
  - AU
  - I0
  - D0
---

# 已验证结论

本文件只记录下列 sealed 或 approved Wash packages 支持的结论。

- 已批准的 7.1 flat7x target 将 I0 设为 640 kbps、D0 设为 1024 kbps。1664 kbps 是两帧组成的 AU aggregate，不是单个 1664 kbps E-AC-3 frame。GTC role projection、D0 algorithm 和 all-AU EMDF fit 仍未闭合，production 保持关闭。证据：`JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1`。
- AU target 要求分开的 I0、D0 inputs 按此顺序组成同一 source-generation，并拒绝 I0-only partial publication。这是 target requirement，不是 legacy durable atomic publication 的证明。证据：`JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1`。
- Rate/frame-size supplement 记录 I0 2560 bytes、D0 4096 bytes 的 target arithmetic，以及有条件的 setup candidates。Setup-to-writer join、D0 `frmsiz` producer、BSI bitrate field 和实际输出仍为 `UNBOUND/NOT_RUN`。证据：`DEE-ATMOS71-I0-D0-RATE-FRMSIZ-BOUNDARY-PARTIAL-V1`。
- E-AC-3 chanmap U16 numeric serialization 使用 emitted slot `15 - numeric_bit_index`；`0x1A00` 的 emitted slots 为 `[3,4,6]`。本地 target-scoped field writer 已重放 Wash 的 16 个 one-hot 与 `0x1A00` 向量；它只验证 numeric field bits。Semantic speaker labels/source-row-to-D0 role join、D0 frame output 与 native execution 保持 `UNBOUND/NOT_RUN`，`production_allowed=false`。证据：`JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1`。
- 较早的 partial transaction package 只观察到不同的 static writer boundaries，未确认它们属于 I0/D0，也未证明 AU 顺序、generation 或 atomic commit。证据：`DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1`。
- Earlier I0 gap notice `JOCFORGE-ATMOS71-I0-FRAME-SYNTAX-GAP-V1` is superseded for the Wash-reviewed user-designated reference structure by `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1`: its I0 is legacy independent AC-3 BSID6 / 2,560 bytes, followed by D0 dependent E-AC-3 BSID16. The Flat7x target's I0 writer must implement that legacy-core reference while its D0 target stays 1,024 kbps / 4,096 bytes. This structure summary does not establish the target writer or authorize changing the existing 5.1 E-AC-3 BSID16 path; target output remains `NOT_RUN`.
- Wash partial package 批准 1024 kbps / 5.1 core 上的 Flat7x reconstruction layout：`JocDownmixProfile` mode0/phase1/internal index3 保持独立，Flat7x 通过 3-bit `joc_dmx_config_idx=1` 表示；profile configuration immutable，不增加 mutable owner。证据：`JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1`。
- Wash CLI-selection addendum 批准可选 `--joc-layout flat7x`，省略时保留既有 1024 kbps / 5.1 profile；Flat7x 的 immutable layout 经 profile resolution 到现有 JOC-header writer。证据：`JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1`。
- Source audit confirms that the selected 1024-kbps full-file CLI path reaches `write_id14_header_for_profile`, and the 5.1 profile retains byte-identical output through the same writer test. `Flat7xI0D0TargetPlan` remains a separate target model; the 1,664-kbps target's D0 frame builder/header connection is the next open boundary.
- `ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF-V1` 支持一个 value-only typed I0=640 / D0=1024 Flat7x AU target plan，以及对候选帧长度、提供的 generation labels 和单一 D0 block-2 EMDF field 元数据的 fail-closed 检查。隔离实现还可把调用方提供且逐帧 CRC 通过的 bytes 拼接为 I0→D0 内存候选；这不解析 frame syntax，也不提供 runtime lineage、writer、audio algorithm 或 publication authority。`production_allowed=false` 持续有效。
- `FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` 允许 target-only staged model：I0/D0 同AU source-generation token 必须相等，PublishedAuIndex 与本地 output-owner generation 各自保留独立域；缺失 required receipt 或 fatal append/finalize failure retire 整个 attempt，恢复要求 programme-cold root。完整 AU count/ledger 可产生 model-only finalize proposal；final exposure、callback-to-generation join 和 GTC output projection 仍开放，production 关闭。
- ETSI TS 102 366 V1.4.1 §4.4.3.58–60 defines `skiple` presence, U9 `skipl` byte count and `skipl×8` skip-field bits. The Flat7x target helper requires a complete nonempty EMDF container and serializes its `skiple=1` form for one D0 block-2 field, accepts 1..511 bytes, preflights before mutation and rejects overflow without continuation. Generic ETSI `skiple=0` syntax is outside this required-container target helper. Five bit-I/O boundary tests pass; frame-cursor/native-owner integration remains `UNBOUND`, and `production_allowed=false`.
- `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` reports a full-track 1,280-kbps I0 legacy AC-3 BSID6 + D0 dependent E-AC-3 BSID16 reference with 2,560-byte frames and D0 block-2 skip-field EMDF. The Flat7x 1,664-kbps target's D0 frame remains 4,096 bytes; target all-AU fit and output remain `NOT_RUN`.
- `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` supplements V1: all 23,033 sample payload-14 headers report U3 `joc_dmx_config_idx=1`, extension index 0 and 15 JOC header objects. This is reference structure evidence; the target's native profile/index writer remains `UNBOUND`.
- `OFFICIAL-ATMOS-DEMO-OAMD-POSITION-WASHED-V1` supplements the same 23,033-AU reference with full-track metadata counts: each of 15 dynamic objects has an absolute room-position/Z update per AU; nine objects use room Z code 0 and six use positive ceiling code 15. `b_enable_elevation`, `object_size_idx`, and resolved object-size height are separate fields. The sanitized fixture is metadata-only; 1,664-kbps target behavior remains `NOT_EVALUATED` and `production_allowed=false`.
- V2 reports payload IDs 11 and 14 with `codecdatae=0`, while ETSI Table 56 requires 1. Keep vendor-compatible observation separate from ETSI-strict conformance; no strict-compliance claim follows.
- Old main：reference-only，不作为实现来源。
