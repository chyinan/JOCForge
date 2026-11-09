---
status: verified
topic: i0-d0-au-composition-boundary
spec: JOCForge
confidence: sealed-wash-package
tags:
  - AU
  - I0
  - D0
  - Wash
---

# I0/D0 AU composition boundary

## 已核验

- 隔离仓库中的 `JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1` manifest SHA-256 为 `0cd5b5e6a1a37961225bfd71bcbf68619baea76c48800449078919acd3499f43`，seal SHA-256 为 `584fdd652ce61102f182f34a133fd083e325cfcde2e9674e5cdaf2c36a576e1a`。Checker 在普通模式和 `python -O` 下均通过。
- Architecture target 要求 I0、D0 使用分开的 typed frame inputs，顺序为 I0 后 D0；一个 AU 必须来自同一 source-generation 并完整发布，拒绝只发布 I0 的 partial 结果。
- 640 kbps I0 与 1024 kbps D0、合计 1664 kbps 的 target rates 来自单独批准的 flat7x profile contract（`JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1`，manifest `7876e3c53e7bda56f60c8d3a691f9082925c6ef14a7899cff21c09f8b1416714`）。这是两帧的 target 值，不是 legacy writer observation。
- 本包只记录 legacy composition 的抽象形态，不证明旧实现曾将 I0+D0 durable publish 为一个 atomic AU。

## 边界注意事项

- 不要把这个抽象 target 与 `DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1` 合并理解。后者没有识别其 writer paths 为 I0 或 D0，也没有证明 AU 顺序、source generation 或 atomic commit。
- 不要从 chanmap bit count、ADM row count 或 JOC carrier count 推导 D0 source-row 或 speaker-role mapping。
- 不要把 target atomic publication 当成 runtime evidence，也不要据此启用 production。

## 未闭合边界与交接

- Runtime source-generation join：`NOT_CAPTURED`。
- D0 source-row-to-semantic-role mapping 和 native D0 audio algorithm：`UNBOUND`。
- Real output 与 W04 acceptance：`NOT_RUN`；`production_allowed=false`。
- 后续规格需要独立绑定 D0 source roles 和 processing contract。本 note 不授权 codec、audio 或 publication 实现。
- 持续维护的链路 handoff：[adm-to-eac3joc-chain-status.md](../architecture/adm-to-eac3joc-chain-status.md)。
