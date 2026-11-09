---
status: verified
topic: washed-7.1-flat7x-conclusions
spec: JOCForge
confidence: sealed-package-backed
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
- Old main：reference-only，不作为实现来源。
