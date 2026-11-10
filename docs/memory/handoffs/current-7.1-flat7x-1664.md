---
status: current
topic: 7.1-flat7x-1664-handoff
spec: JOCForge
confidence: package-backed-target
last_updated: 2026-11-11
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

## 2026-11-10 Love Song Flat7x run

当前实测目标是一个独立的 1024 kbps / selected 5.1 core / Flat7x layout attempt，输入绑定到 E-275 同源的 `D:\Love Song 方大同.wav`，SHA-256 `34ecda6afa708aaf436930b86cc8d120a73faff2847088be975b10a22039f8b5`，36-channel PCM24 / 48 kHz。E-521 prelaunch 经 Wash 登记并放行；实际命令附加 `--joc-layout flat7x`，运行二进制来自提交 `5431a6812a041562d938314fd4ec2dd2377fad80`。

单次 attempt `1794325737775022700` 于 2026-11-10 15:51:27 UTC 以 exit code 2 结束。日志记录 Meter 完成后，MAIN g=0、g=1 priming 和 source s=0、s=1 suppressed，随后停止于 `F14/F14K source stage: MetadataOwner(CurrentEmdfValidation)`。最终 `.ec3` 不存在，未发现 `.ec3.partial`；receipt SHA-256 为 `f9d73af6b7b1af6c6932f32e5e70d2c793070f3c55f86f7495531bffe574b449`。

FFprobe 外层 stream metadata、JOC internal index/object-count/EMDF、official-demo structure match 与 listening acceptance 均为 `NOT_RUN`。没有读取或保存 PCM/压缩 payload。Wash 已收到完整 receipt，当前等待其将失败边界加入统一时间线并指明下一条 evidence step；此 attempt 不自动重试。

## CurrentEmdfValidation future diagnostic

F13 `MetadataOwnerError::CurrentEmdfValidation` now carries a non-payload typed predicate reason while preserving its stable error code and fail-closed decision. The E-521 receipt predates this discriminator, so its exact failing guard remains `UNBOUND`; future authorized runs can retain the reason in the source-stage error receipt without recording EMDF bytes or hashes.

## 1664 typed-target handoff

Wash handoff file `ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF-V1/IMPLEMENTER-HANDOFF.md` was independently hash checked at `4c8d964b8a91b67c4158e0141ec8dcfbf91f3245e00a85138188bc5389f104b4`. It defines an architecture-only I0=640 kbps / D0=1024 kbps Flat7x AU plan, with `production_allowed=false`. The local value-only `Flat7xI0D0TargetPlan` and `validate_flat7x_au_candidate` encode the approved two-frame order, target extents, reconstruction metadata, single D0 block-2 EMDF field cap, and whole-AU rejection policy. `compose_flat7x_au_byte_sequence` accepts caller-provided slices after target extent and per-frame CRC checks and concatenates them in I0→D0 order. The additive `Flat7xAuOutputOwner` requires the F13 metadata proposal and separate I0/D0 integrity receipts before creating a pair-aware ledger entry. Its move-only proposals carry F13 and frame-integrity receipt references. Fatal validation or finalize failure returns an attempt-retirement value; finalize requires the expected AU count and complete ledger. The separate IO adapter source consumes the typed proposal, writes/syncs/reads back caller-owned staging and can create the receipt candidate; temporary-file tests exercise its byte IO helper, while the full F13/Integrity receipt-bundle adapter call remains unrun. No rollback, final-file exposure, GTC registration or production binding exists. It does not parse frame syntax or prove source ownership. Candidate generation values remain caller-supplied labels; this does not create runtime source lineage. Nine target-plan/sequence, six owner, and three temporary-file IO helper tests pass; the 5.1 resolver and six-rate frame/output/profile checks pass. No frame writer, production admission, or real product output capability is present. D0 writer ownership, ADM role join, D0 audio algorithm and all-AU EMDF fit remain `UNBOUND`.

## 2026-11-11 F15 receipt-pair input-only replay

Wash package `FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1` was checked against manifest SHA-256 `1034ef415113b666acc5aefaa3207230040f05d9ebc05a33933bffc50b4ff088` and package-seal SHA-256 `16d925352e9b197e016b2554f9c2ae609e8f8890e3dca81c4eb591f737b41210`. Its vector drives the `Flat7xAuOutputOwner` receipt gate with synthetic typed F13/I0/D0 values and in-memory CRC-valid I0/D0 frames. The positive pair produces the expected 6,656-byte ledger row; identity mismatch, wrong carrier block and C=512 retire the attempt. The fixture constructs typed test projections without invoking the F13 parser or F12 Integrity owner stage. Parser-valid container acceptance, owner-stage receipt production, runtime callback/source-generation join, GTC registration, final exposure and production remain open; `production_allowed=false`.

## 下一步

- Wash `FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` (manifest `18de94b6b18a13222df8e9eae018f77c49fa735007869b99255b97a1cb5ae0d1`, seal `8879c0417bf60db4d8dc9dec1fbbeccd884978af6460f304cfb8034052e4d3ee`) authorizes the target-only attempt/fail-stop/pair staging model. Local owner requires F13 metadata plus ordered I0/D0 integrity receipts, keeps source label, PublishedAuIndex and owner generation separate, returns retirement on fatal failure, and can create a finalize proposal only after complete AU count/ledger validation. It does not enable canonical GTC, final expose or production. Positive F13/Integrity pair integration and callback-generation join remain open.

- 后续实现须等待开放 callback/source owner join、GTC output projection/final-exposure receipt、I0/D0 syntax/audio role gates 和 all-AU fit 的批准证据；保持 1664 target 的生产路径关闭。任何编码运行需要单独的 input/output/temp preflight、Wash timeline registration 和 GO。
