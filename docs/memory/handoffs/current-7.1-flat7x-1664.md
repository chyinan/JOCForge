---
status: current
topic: 7.1-flat7x-1664-handoff
spec: JOCForge
confidence: package-backed-target
tags:
  - handoff
  - AU
  - I0
  - D0
---

# 当前 7.1 flat7x / 1664 handoff

## Target

批准的 target 使用分开的 I0、D0 frames：先 I0 640 kbps，再 D0 1024 kbps。1664 kbps 是一个 AU 中两帧的 aggregate rate，不是单个 1664 kbps frame。AU target 要求同一 source-generation 校验和完整发布；拒绝 I0-only partial result。

## 隔离仓库中已验证的内容

- Profile target、rate/frame-size arithmetic、AU composition requirement 和 field-local chanmap U16 serialization 各有独立的 sealed Wash supplement。Package identity 和剩余边界见 [evidence-index.md](../shared/evidence-index.md)。
- Chanmap target rule 只涉及 numeric field serialization；semantic speaker labels 来自分开的 profile contract。
- 较早的 partial native transaction package 没有证明 I0/D0 writer identity、target order、same-generation lineage 或 atomic publication。

## 开放边界

- D0 source-row-to-semantic-role mapping：`UNBOUND`。
- Native D0 audio algorithm：`UNBOUND`。
- Runtime source-generation join：`NOT_CAPTURED`。
- Legacy durable atomic publication：未证明。
- Real output 和 W04 acceptance：`NOT_RUN`。
- `production_allowed=false`。

## 下一步

等待能闭合 D0 source-role 和 processing contract 的 sanitized Wash material。在此之前，不从 chanmap set bits 或 partial legacy transaction package 推断这些规则，也不启用 production。
