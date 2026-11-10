---
status: current
topic: 7.1-flat7x-1664-handoff
spec: JOCForge
confidence: package-backed-target
last_updated: 2026-11-10
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
- I0 frame syntax 仍有独立 gap：官方摘要的 `legacy_independent` 未给 exact BSID；selected prefix 当前是 BSID16/`strmtyp=0`，与官方 I0 的关系 `UNBOUND`。OpenJOC parser capability 仅提供 parser 范围信息；selected prefix 保持当前值，直到 official I0 exact BSID 有新证据。
- 已按新的 partial Wash package，在现有 1024 kbps / 5.1 core profile 上实现 immutable `JocReconstructionLayout::Flat7x` 与 3-bit `joc_dmx_config_idx=1` header field。mode0/phase1/internal index3 和 5.1 coefficient input path保持独立；profile-aware writer 的 field-order 和 baseline byte tests 通过。
- CLI 可选项 `--joc-layout flat7x` 已接入 resolver 与 JOC header writer；省略该项维持 ExistingSelectedFivePointOne。
- 较早的 partial native transaction package 没有证明 I0/D0 writer identity、target order、same-generation lineage 或 atomic publication。

## 开放边界

- D0 source-row-to-semantic-role mapping：`UNBOUND`。
- Official I0 exact BSID：`NOT_CAPTURED`；selected prefix equivalence：`UNBOUND`。
- Native D0 audio algorithm：`UNBOUND`。
- Runtime source-generation join：`NOT_CAPTURED`。
- Legacy durable atomic publication：未证明。
- Real output 和 W04 acceptance：`NOT_RUN`。
- Flat7x complete stream structure、official-demo acceptance 与 7.X audio rendering：`NOT_RUN`。
- 1664 kbps I0/D0 AU target 的 `production_allowed=false`。

## 下一步

Love Song 输入沿用已成功收据绑定的 `D:\Love Song 方大同.wav` / SHA `34ecda6afa708aaf436930b86cc8d120a73faff2847088be975b10a22039f8b5`。Flat7x output/temp 已规划为唯一空路径。启动等待 Wash 将更新后的 selector/parser prelaunch receipt 写入统一时间线并返回 continue；随后只按字段范围核验输出结构。
