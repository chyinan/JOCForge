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
- E-AC-3 chanmap U16 numeric serialization 使用 emitted slot `15 - numeric_bit_index`；`0x1A00` 的 emitted slots 为 `[3,4,6]`。此规则不推导 semantic speaker labels，也不绑定 ADM source rows 到 D0 roles。Native execution 为 `NOT_RUN`。证据：`JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1`。
- 较早的 partial transaction package 只观察到不同的 static writer boundaries，未确认它们属于 I0/D0，也未证明 AU 顺序、generation 或 atomic commit。证据：`DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1`。
- I0 syntax identity 尚未闭合：官方结构摘要只标注 `legacy_independent`，exact BSID 为 `NOT_CAPTURED`；隔离仓库当前 selected prefix 写 E-AC-3 BSID 16/`strmtyp=0`，与官方 I0 的等价性为 `UNBOUND`。OpenJOC parser capability 只描述 validator/parser 能力，不证明官方输出格式或 encoder 行为；此 gap notice 不授权修改 prefix。证据：`JOCFORGE-ATMOS71-I0-FRAME-SYNTAX-GAP-V1`。
- Wash partial package 批准 1024 kbps / 5.1 core 上的 Flat7x reconstruction layout：`JocDownmixProfile` mode0/phase1/internal index3 保持独立，Flat7x 通过 3-bit `joc_dmx_config_idx=1` 表示；profile configuration immutable，不增加 mutable owner。证据：`JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1`。
- Wash CLI-selection addendum 批准可选 `--joc-layout flat7x`，省略时保留既有 1024 kbps / 5.1 profile；Flat7x 的 immutable layout 经 profile resolution 到现有 JOC-header writer。证据：`JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1`。
- `ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF-V1` 支持一个 value-only typed I0=640 / D0=1024 Flat7x AU target plan，以及对候选帧长度、提供的 generation labels 和单一 D0 block-2 EMDF field 元数据的 fail-closed 检查。隔离实现还可把调用方提供且逐帧 CRC 通过的 bytes 拼接为 I0→D0 内存候选；这不解析 frame syntax，也不提供 runtime lineage、writer、audio algorithm 或 publication authority。`production_allowed=false` 持续有效。
- Old main：reference-only，不作为实现来源。
