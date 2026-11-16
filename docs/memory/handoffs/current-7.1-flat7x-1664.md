---
status: current
topic: 7.1-flat7x-1664-handoff
spec: JOCForge
confidence: package-backed-target
last_updated: 2026-11-15
tags:
  - handoff
  - AU
  - I0
  - D0
---

## 2026-11-14 candidate 状态

Love Song 方大同.wav 已确认是 36 声道、48 kHz、PCM24 ADM BWF，带 `axml` 与 `chna`。同源 F06ND BSI 输入和 attempt-local `.candidate.ec3` 捕获路径已接入；写入采用 create-new、完整 F15 pair ledger、精确读回/hash、文件与目录同步。失败时保留部分候选并写 `.failure.txt`；候选保持 `production_allowed=false`，Programme DONE root 不安装。

按用户直接指令完成的 Release run：PID `16072` / attempt `1794647185411283100`，输出为 `D:\JOCForge-attempts\love-song-flat7x-1664-release-20261114-170625\love-song-flat7x-1664.candidate.ec3`；8,417 AU / 56,023,552 bytes，F15 stage readback/sync receipt SHA-256 `6101459cc6b04cda1b192cf7add7a8dc2e0775c8f4965b08e85040bea19e3b0c`。OpenJOC parse/decode/render 与 W04 尚未运行。run timeline/Fresh GO 未预先由 Wash 登记，实际情况已回报 Wash；candidate 内容未手工读取。

Wash 元数据包 `love-song-flat7x-object-to-joc-row-metadata-only-wash-v1`（manifest `ec88b6cd216857d5fb758a892e0d8540a51d7fcea78f8d2566d68d6ac284dd30`；seal `188051a489ffcc03a8894c781208361a0664b64bbd639a00be27cf136b669812`）确认输入有 26 个通过 CHNA/AXML UID+pack 显式连接的 ADM Object tracks。它们到 target 15 JOC objects/rows 与七个 reconstruction carriers 的 crosswalk 仍为 `UNBOUND`；候选试跑不作为 object/spatial compliance 或 listening parity 证据。

目标候选不与 DEE 等价；OpenJOC parse/decode/render 与 W04 保持 `NOT_RUN`，5.1 路径保持不变。

# 当前 7.1 flat7x / 1664 handoff

## Target

批准的 target 使用分开的 I0、D0 frames：先 I0 640 kbps，再 D0 1024 kbps。1664 kbps 是一个 AU 中两帧的 aggregate rate，不是单个 1664 kbps frame。AU target 要求同一 source-generation 校验和完整发布；拒绝 I0-only partial result。

## 2026-11-14 当前接线进度

Commit `35503d6d404dbafbdbdddc05cbe1b8abaefa4043` 已将 Flat7x I0+D0 durable append 与 programme callback parent replacement 接入 CLI，并推送到 `experimental/independent-encoder`。

当前工作树新增同源 D0 BSI producer input：`bind_flat7x_d0_bsi_source_inputs` 从 `NoneDrcOwnerAfterProposal` 和 prepared authored downmix 取得 dialnorm、RF `compr`、六个 line DRC codes 与 mixing values，并保留 shared-analysis identity；CLI 在 D0 frame 组装前与 pair source role receipt 比对。它复用现有 F06ND/F12 field projection 和 `FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1`，没有新增 audio math。selected 5.1 path不变。

Wash 已封存官方 AU0 D0 的两字节 CI16 field `0x01 0x10`，实现侧已将其按 byte array 贯通 D0 BSI source bridge 与 required `Flat7xD0Ci16Addbsi` writer field。Parser shape 为 byte0 bit0 extension-present、byte0 bits1..7 reserved zero、byte1 U8 complexity index 16；它与 Type-A `complexity16` 的 join 仍 `UNBOUND`。全输入 CLI、OpenJOC render 与 W04 尚未运行，`production_allowed=false` 继续保留。

## 隔离仓库中已验证的内容

- Profile target、rate/frame-size arithmetic、AU composition requirement 和 field-local chanmap U16 serialization 各有独立的 sealed Wash supplement。Package identity 和剩余边界见 [evidence-index.md](../shared/evidence-index.md)。
- Chanmap target rule 只涉及 numeric field serialization；semantic speaker labels 来自分开的 profile contract。
- `JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1` 的 numeric helper 保留在 `codec::eac3_channel_map`。后续 Wash supplement `eac3-dependent-chanmap-field-writer-v1` 增加同模块的 pair helper：先写 `chanmape=1`，再写 typed 16-bit MSB-first `chanmap`，完整预检 17 bit。16 个 one-hot、target `0x1A00`、start bit 5 misalignment 与 16-bit capacity reject vectors 本地重放通过。helper 不接入完整 D0 BSI/frame/AU path。
- The pair supplement's `acmod=6,lfeon=0` is an independent validation-vector context; the Flat7x D0 target tuple is `acmod=5,lfeon=0` as confirmed by the fixed-prefix contract. The field-pair helper consumes neither context.
- Wash package `eac3-dependent-d0-fixed-prefix-v1` adds `codec::frame_builder::write_flat7x_d0_fixed_prefix`. It preflights and emits exactly the target's 45-bit prefix, ending at bit 45; the caller owns `dialnorm` and remaining BSI. The chanmape/chanmap pair stays a later, separate BSI field group. Its corrected capacity vector describes a 5-byte / 40-bit buffer and verifies no partial write. The helper does not compose a full D0 frame or enter production.
- Wash package `eac3-dependent-d0-bsi-segment-v1` adds `codec::frame_builder::write_flat7x_d0_bsi_through_chanmap`. It composes prefix + caller-supplied dialnorm + compre/present optional compr + chanmape/chanmap and returns before caller-owned `mixmdate`. The full segment preflight is 68 bits without compr and 76 with compr. dialnorm 0 and 32 reject before mutation. No XML DRC translation, compression algorithm, complete frame or production path is introduced.
- Wash package `eac3-dependent-d0-mixing-metadata-v1` adds `append_flat7x_d0_mixing_metadata_and_infomdate` after the parent cursor 68/76. It serializes explicit optional mixing levels, six optional block values, and infomdate; it returns immediately after infomdate and preserves later information metadata for the caller. No XML mapping or audio algorithm is added. The package capacity vector says 16 bits remain at cursor68; the byte-backed `FrameBitWriter` exposes 12 or20 there, so local atomicity testing uses 12 and awaits Wash's vector correction.
- Wash package `flat7x-i0-ac3-header-segment-v1` adds `codec::frame_builder::write_flat7x_i0_ac3_header_through_compression` for the independent I0 legacy AC-3 path. It emits only the header through optional `compr`, accepts frmsizecod 36/37, and returns at the caller-owned langcode boundary after 62/70 bits. CRC1 and the DRC fields are caller-supplied values; this helper computes no CRC and maps no XML DRC profile.
- I0 reference value is provided by `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1`: the user-designated sample uses legacy independent AC-3 BSID6 at 2,560 bytes. The target now has a synthetic caller-span frame assembler with generated CRC1/CRC2 and zero-residual receipts. Audio-block production and the typed source callsite remain open. The selected 5.1 prefix remains E-AC-3 BSID16/`strmtyp=0`, unchanged.
- 已按新的 partial Wash package，在现有 1024 kbps / 5.1 core profile 上实现 immutable `JocReconstructionLayout::Flat7x` 与 3-bit `joc_dmx_config_idx=1` header field。mode0/phase1/internal index3 和 5.1 coefficient input path保持独立；profile-aware writer 的 field-order 和 baseline byte tests 通过。
- 已核对的 CLI writer 链为 `run_with_data_rate_and_joc_layout` → `run_worker`/`run_inner` profile resolution → callback `stage_source_frame` → `prepare_main_metadata_bodies_for_callback` → `stage_selected_joc_source_with_transaction_for_profile` → `stage_joc_source_with_transaction_for_profile` → `stage_joc_payload_for_profile` → `write_id14_header_for_profile`。该调用链的对象是现有 selected 1024 kbps full-file profile；`--joc-layout flat7x` 将 index 1 送到 ID14 writer，省略选项的 5.1 writer bytes 与既有 writer 完全一致。
- 上述调用链对应 selected 1024 kbps profile；`Flat7xI0D0TargetPlan` 保持独立的 1,664-kbps target model。target D0 侧已有独立的 `d0_emdf_skip_field` helper；下一连接点是由 D0 frame composition 调用 profile-aware ID14 producer，并传入 1024-kbps D0 context。selected 5.1 carrier 路径与 D0 composition 分开。
- 较早的 partial native transaction package 没有证明 I0/D0 writer identity、target order、same-generation lineage 或 atomic publication。

## 开放边界

- D0 source-row-to-semantic-role mapping：`UNBOUND`。
- Flat7x I0 target fields now select the user-designated reference's legacy AC-3 BSID6; typed header/BSI/frame assembly exists, while six-block audio source lineage and production admission remain open. The existing selected 5.1 E-AC-3 BSID16 path remains separate.
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

## 2026-11-11 D0 skip-field serializer

The approved Flat7x package fixes a single D0 zero-based block-2 skip field, C<=511 bytes, no continuation and whole-AU fail-closed overflow. Wash confirmed ETSI TS 102 366 V1.4.1 §4.4.3.58–60 syntax. The target-only `codec::emdf_skip_field::write_flat7x_d0_block2_skipfld` requires a complete nonempty container and writes the present form `skiple=1`, U9 `skipl=C`, followed by C bytes after delta-bit allocation and before mantissas. Missing/empty target input, C=512, and insufficient capacity all return before changing cursor/frame bytes. Generic ETSI `skiple=0` remains outside this target-required-container API. The helper leaves D0 chanmap, JOC object count and reconstruction carriers separate. Five bit-boundary tests pass.

The caller owns the block-two insertion offset through `CallerOwnedUnboundD0Block2InsertionPoint`. `D0FrameBitAssembler` now consumes the helper at the layout-computed block-2 field span for a synthetic exact-size frame; that connects the field writer to a caller-owned block/frame interface while the native block-2 cursor, D0 BSI/audio owner, source/generation lineage, all-AU fit and product output remain `UNBOUND/NOT_RUN`; `production_allowed=false`.

## 2026-11-12 Flat7x D0 skipfld structure addendum

Wash package `FLAT7X-D0-SKIPFLD-STRUCTURE-V1` was independently checked normal and with `python -O`; manifest SHA-256 `1b14343db4128bda45c254a024de2662ac59516e2e83aa7d3191e3c4d80ca1c3`, seal-file SHA-256 `7c31c485368c3755de23f7fee981a141c3607eef86d7110a4cdf1661c6a15024`. It reiterates one D0 block-2 skip field: `skiple=1`, U9 `skipl=C` bytes, then exact opaque container bytes; C must be 1..511, no fragmentation, whole-target fail-closed overflow. The official 1,280-kbps sample observes C_total/skipl 301..377; target remains 640+1024=1664 kbps. The existing helper already writes the exact field, and the synthetic D0 frame assembler passes it the caller-owned block-2 container at its layout-computed insertion span. C511 is `FIT_EXACT` arithmetic/writer evidence only; OpenJOC full-container C511 acceptance remains `NOT_RUN`, and native block cursor/source owner remain unbound.

## 2026-11-11 official Atmos reference structure

Wash package `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` passed both checker modes and is registered at manifest `f7f8630c3f18e990324f013e70486c561de6816a844f852f18f03a82ce28fbfc` / seal `2d883e278bb83fa0d617610f94b8a87fa5849ac7bf19bdf08818d8d30d50740e`. Its 23,033 packet summary reports I0 legacy independent AC-3 BSID6 / 2,560 B followed by D0 dependent E-AC-3 BSID16 / 2,560 B, 1,280 kbps aggregate. D0 uses `chanmap=0x1A00`, a two-byte addbsi/CI16 record, and block-2 EMDF IDs 11/14 with observed skip-field lengths 301–377 B. The user target remains I0 640 + D0 1,024 = 1,664 kbps, with a 4,096-byte D0 frame; the sample's 640-kbps D0 and target are distinct. No media/audio bytes are included, and the package keeps `production_allowed=false`.

Wash addendum `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` (manifest `c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a`, seal `0fb4c7f8a7a49488a97586b7e3eb6febc0542cf0044569fe37e65dc07d85757d`) reports `joc_dmx_config_idx=1`, extension index 0 and 15 JOC header objects on all 23,033 AUs. This binds the reference sample's profile index; it does not establish a native target writer or profile-index 1 on a new 1,664-kbps output. Payload IDs 11/14 have `codecdatae=0` while ETSI Table 56 requires 1; vendor-compatible observations and ETSI-strict failure remain separate, and no strict-conformance claim is made.

The new reference identifies the intended Flat7x I0 structural value as legacy AC-3 BSID6. The profile-specific I0 writer path still needs its own implementation; preserve the existing 5.1 E-AC-3 BSID16 path. The D0 skip-field helper is called by the synthetic caller-span frame assembler; native block-cursor and runtime owner binding remain unbound.

## 2026-11-11 official Atmos OAMD position reference

Wash package `OFFICIAL-ATMOS-DEMO-OAMD-POSITION-WASHED-V1` is registered at manifest `c550a0536ed4752b46872c674fcabc8ad8828b7eb69e17e6fb4a0d082ee52ef8` / seal `906f48f17f47ef372d339bebe598931a933e6dc132452862ab630fce4582c5f5`. Its normal and `python -O` checkers pass. The user-designated 1,280-kbps sample has 23,033 AUs, one bed/ISF slot and 15 dynamic-object slots. Each dynamic object has one absolute room-position/Z update per AU; nine objects use Z code 0 and six use positive ceiling code 15. The `b_enable_elevation` zone constraint is true on all 345,495 updates. Separately, `object_size_idx=0` on all 345,495 updates and resolved object-size height is zero across 345,510 states.

The sanitized target reference fixture is `crates/jocforge-domain/tests/fixtures/flat7x_metadata/official_demo_oamd_position_reference.json`. It keeps position Z, the elevation-zone constraint, and object-size index/resolved height in separate fields. It contains aggregate counts only; coordinates, source-row mapping, target metadata recipe, and encoding behavior remain open. The sample is 1,280 kbps while the 1,664-kbps target is `NOT_EVALUATED`; `production_allowed=false`.

## Flat7x reuse and target-native gap matrix

| Component | Existing reusable structure | Flat7x target-specific requirement | Evidence and status | 5.1 regression gate |
|---|---|---|---|---|
| Main sealed reference | Ordered I0→D0 AU shape and `OwnershipPlan` architecture shape | No frame syntax, rate allocation, BSI value, or D0 writer is inherited from the reference | Architecture shape only; runtime/writer identity `UNBOUND` | No 5.1 source is reused or changed |
| Old map / split-rate / legacy D0 | None for the new target | Old `chanmap=0x0200`, old 1,024+640 split and legacy D0 writer stay out of the Flat7x row | Legacy/reference paths remain separate evidence | Existing 5.1 map and writer behavior stays fixed |
| Isolated 5.1 path | Six-rate resolver, selected F12 E-AC-3 frame writer, `OutputOwner`, and durable atomic sink remain the independent baseline | Flat7x additions stay additive; no global frame-prefix or rate-row rewrite | Current 5.1 path remains implemented and locally regression-tested | Run `selected_frame_trial::tests::supplied_absent_payload_plan_builds_only_a_complete_keyless_scratch_frame`, `output_owner_rate_profile`, and `prepared_adm_input_preserves_each_washed_rate_through_profile_and_frame_size` |
| I0 | Existing bitio primitives, the AC-3 header segment, typed BSI-tail serializer, caller-span frame assembler and ETSI CRC generator | Legacy independent AC-3 target BSID6, 640 kbps, 2,560 bytes | The BSI-tail writer serializes langcode, audprodie, copyright/origbs, optional U14 timecodes and bounded addbsi after header cursor 62/70; acmod7 omits dual-mono fields. The assembler stages that opaque tail with six caller audio-block spans and optional auxdata, generates CRC1/CRC2, and verifies synthetic zero residuals. Audio blocks, source binding and product output remain open. | Keep selected 5.1 E-AC-3 BSID16 path unchanged |
| D0 | Existing bitio primitives, fixed-prefix/BSI/mixing/information segment writers, typed BSI composition, six-block layout proposal, CRC16-generating frame-bit assembler, and separate skip-field serializer | Dependent E-AC-3 BSID16, 1,024 kbps, 4,096 bytes, `chanmap=0x1A00`, one block-2 complete EMDF skip field | Prefix/BSI/metadata structure and synthetic six-block layout have local tests. The frame-bit assembler copies only caller-supplied opaque spans and block-two EMDF bytes, then generates CRC2 over frame[2:4094], writes it big-endian at frame[4094:4096], and verifies a zero residual over frame[2:4096]. It returns a typed CRC2 integrity receipt. Vectors remain `SYNTHETIC_NOT_REAL_OUTPUT`; no audio/mantissa/BAP behavior or playable output is established. Full frame acceptance, native block cursor, AU/source role and production remain open. | Existing 5.1 header path remains unchanged |
| JOC reconstruction and object metadata | Existing profile-aware ID14 header writer plus additive Flat7x target-plan header adapter; metadata/source wiring stays separate | Flat7x reconstruction profile index1 is distinct from downmix mode0/phase1/internal index3; seven reconstruction carriers, 15 JOC object/reconstruction rows, complexity16; ADM physical input rows remain N and separate | V2 reports sample index1, extension0 and 15 header objects. The selected 1024-kbps CLI path writes index1 and preserves 5.1 byte equality. The new target-only ID14 adapter requires the approved Flat7xI0D0TargetPlan and semantic object_count=15, which the writer encodes as U6 14; complexity16 is not part of ID14. The 1,664-kbps full output, D0/audio/metadata source wiring, and independent parser/W04 acceptance remain open. | Mode0/phase1/internal index3 and existing 5.1 header path remain unchanged |
| AU transaction and output | Existing target-specific pair plan/ledger and staging contract, plus a separate opaque I0→D0 byte composer | Same-source I0+D0 receipts, GTC projection, final commit/exposure and all-AU EMDF fit | The byte composer checks target extents/order/capacity only; runtime generation join remains `UNBOUND`, and no CRC/BSID/source identity is inferred. Output/W04 remain `NOT_RUN`. | Existing 5.1 sink remains unchanged |
| EMDF / validation mode | Existing typed keyless CRC/checksum primitives and F15 whole-AU receipt gate | Complete IDs 11/14 container in one D0 block-2 skipfld, C=1..511, no continuation | V1 observes sample C=301..377; V2 reports sample `codecdatae=0` for IDs 11/14 while ETSI Table 56 expects 1. Vendor-compatible observation and ETSI-strict failure stay separate; target 1,664-kbps all-AU fit `NOT_RUN` | EMDF changes stay outside the 5.1 F12 writer |

The official demo sample is 1,280 kbps; the implementation target remains 1,664 kbps. These are separate rows and extents.

## 下一步

- Wash `FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` (manifest `18de94b6b18a13222df8e9eae018f77c49fa735007869b99255b97a1cb5ae0d1`, seal `8879c0417bf60db4d8dc9dec1fbbeccd884978af6460f304cfb8034052e4d3ee`) authorizes the target-only attempt/fail-stop/pair staging model. Local owner requires F13 metadata plus ordered I0/D0 integrity receipts, keeps source label, PublishedAuIndex and owner generation separate, returns retirement on fatal failure, and can create a finalize proposal only after complete AU count/ledger validation. It does not enable canonical GTC, final expose or production. Positive F13/Integrity pair integration and callback-generation join remain open.

- 后续实现须等待开放 callback/source owner join、GTC output projection/final-exposure receipt、I0/D0 syntax/audio role gates 和 all-AU fit 的批准证据；保持 1664 target 的生产路径关闭。任何编码运行需要单独的 input/output/temp preflight、Wash timeline registration 和 GO。

## 2026-11-11 D0 informational BSI/addbsi suffix

Wash package `EAC3-DEPENDENT-D0-INFO-ADDBSI-SEGMENT-V1` (manifest `e8f1908d3572571c4dcb19aedd898b0e068464d6921ed54569fd8df4cb3a7cdb`, seal `d94fd72b890c522849cca6380f7630eddf1637764fda09807a61aa8ce3385f5d`) supplies three exact suffix vectors and field/length/capacity rejection inputs. The local `append_flat7x_d0_info_and_addbsi` helper starts at the parent mixing/infomdate cursor, emits explicit caller-owned informational fields, then `addbsie` and optional `addbsil` plus opaque bytes. It admits only the selected D0 context with a present `chanmap=0x1A00`; missing and wrong map tests verify rejection without byte or cursor changes. Normal and optimized package verification passed; local vector and atomic-rejection tests cover the suffix only. The synthetic `0x12 0x34` bytes are opaque test input. JOC payload interpretation, audio, CRC, complete D0 frame/AU, source binding and production remain open; `production_allowed=false`.

## 2026-11-11 typed D0 BSI structure composition

`write_flat7x_d0_bsi_structure` composes the registered fixed-prefix/BSI, mixing/infomdate, and information/addbsi segment writers in that order behind one typed request. `measure_flat7x_d0_bsi_structure` validates every component and calculates its complete extent while inspecting opaque addbsi bytes by length only. The writer preflights caller capacity before staging or copying. Local tests compare the composed result byte-for-byte with the ordered component writers and compare measured/emitted extents; invalid components, short capacity, and nonzero start cursor leave caller bytes/cursor unchanged. This is not a Wash-provided combined expected-output vector and does not connect audio, complete frame/AU, GTC or production; the exact target still requires the present `chanmap=0x1A00`.

## 2026-11-11 synthetic six-block D0 layout proposal

`propose_flat7x_d0_six_block_layout` consumes six ordered audio bit lengths and a block-two skip-field descriptor with a caller-owned/unbound insertion offset. It measures the D0 BSI structure without inspecting opaque addbsi contents beyond their length, requires exactly one complete block-two field, rejects any field on another block, enforces payload C=1..511 and a total target span no larger than 4096 bytes, and returns bit-span positions only. Audio inputs carry lengths without payload bytes; no PCM, audio, mantissa, CRC, frame, AU, GTC or production output is produced. Synthetic cases cover exact spans, C=511, C=512, missing/duplicate/misplaced fields, out-of-span offsets, target overflow and arithmetic overflow. The new layout does not bind the insertion point to a native block cursor or continuation into block 3.

## Historical 2026-11-11 caller-supplied D0 frame-bit assembler

`assemble_flat7x_d0_frame_bits` stages the approved 4096-byte D0 frame using caller-supplied opaque audio bit spans for six blocks, one block-two EMDF container bracketed by caller-supplied pre/post spans, and caller-supplied CRC2 U16. The explicit block-two insertion offset remains caller-owned/unbound. It requires the BSI, six spans, single skipfield, and CRC2 to total exactly 32768 bits; source extent, output capacity and placement checks precede the single caller-writer commit. Synthetic test patterns are explicitly `SYNTHETIC_NOT_REAL_OUTPUT`; they do not establish playable output or audio/DEE equivalence. No audio algorithm, mantissa/BAP generation, CRC calculation, block3 continuation, AU/GTC or production call is connected.

## 2026-11-11 Flat7x target ID14 header adapter

`write_id14_header_for_flat7x_target` requires the approved `Flat7xI0D0TargetPlan` and semantic object count 15, then invokes the existing profile-aware ID14 writer with the Flat7x reconstruction index 1. The writer's approved U6 count-bits field is `15 - 1 = 14`; mode0/phase1/internal index3, complexity16, seven reconstruction carriers, 15 reconstruction rows and ADM physical input rows remain distinct. The header-only input has no body field. Missing/invalid target contracts and object-count mismatch reject without changing writer state; the existing 5.1 header byte-equality regression remains in the same suite. This adapter emits no D0 audio or complete 1,664-kbps AU/frame and does not close production or parser/W04 acceptance.

## 2026-11-11 opaque I0→D0 AU byte composer

`write_flat7x_i0_d0_access_unit_bytes` accepts typed opaque I0/D0 frame byte spans, checks the approved 2560/4096/6656-byte extents, requires writer cursor zero and preflights all 6656 bytes. It stages I0 followed by D0 and commits once, then returns a noncommitting composition-layout receipt containing only byte spans/extents. BSID, CRC, source-generation identity and AU semantic metadata remain outside this slice. The vector uses synthetic fill bytes marked `SYNTHETIC_NOT_REAL_OUTPUT`; the selected 5.1 path is unchanged.

## 2026-11-12 opaque Flat7x elementary-stream sequence composer

`codec::elementary_stream_composer::compose_flat7x_elementary_stream` takes caller-provided, already-composed opaque AU byte spans in sequence order. It rejects empty input and any AU not exactly 6656 bytes, checks count/byte/bit arithmetic, requires a byte-aligned writer cursor and preflights the complete extent/cursor range before exact private staging. One writer call appends all AUs in order and returns a noncommitting composition-layout receipt. The input-only vectors use synthetic fills to verify two-AU ordering, overflow/empty/extent/capacity/alignment rejection, and no caller mutation. `production_allowed=false`; no file/GTC/product encode/audio path is connected.

The synthetic structural integration test feeds `assemble_flat7x_d0_frame_bits` output into the I0→D0 AU composer and then into a two-AU stream. AU and stream offsets are derived from their receipts and checked against the full 4096-byte D0 span; the generated CRC2 tail bytes remain unchanged at both stream positions. This test establishes structural copy lineage for synthetic bytes only.

## 2026-11-12 Flat7x elementary-stream temp staging

`Flat7xElementaryStreamStageProposal` now consumes typed F15 `Flat7xAuFinalizeProposal` and verifies ordered AU ledger offsets, extents and I0/D0/AU hashes before staging. The IO receipt candidate echoes the same attempt ID/owner generation; proposal completion checks those identities and exact readback SHA. The separate `jocforge-io` adapter writes only to a caller-owned create-new temp `File`, loops over short writes, rejects zero progress, flushes/syncs and performs bounded readback. It has no final path, rename, GTC or product connection; caller abandons partial staging on error. A synthetic F15 ledger test verifies the structural typed join, while live F13/Integrity receipt production, source callback, parent commit and public File-adapter invocation remain `NOT_RUN`; `production_allowed=false`.

## 2026-11-12 E-591 first-output preflight target

Wash预检将目标ADM身份绑定为 `D:\Love Song 方大同.wav`，size `1,396,237,802` bytes，SHA-256 `34ecda6afa708aaf436930b86cc8d120a73faff2847088be975b10a22039f8b5`。candidate output为 `D:\Programs\JOCForge-isolated\.test-tmp\love-song-flat7x-1664-20261112-0139.ec3`，temp目录为 `D:\Programs\JOCForge-isolated\.test-tmp\love-song-flat7x-1664-20261112-0139-tmp`；timeline entry `E-591` 记录预检时两个destination均不存在。此绑定只供未来Fresh preflight使用：当前 `Fresh GO=NOT_RECEIVED`、`production_allowed=false`、编码未启动；复用前需重核路径/hash和destination空闲状态。


## 2026-11-12 D0 CRC2 generation

The additive Flat7x D0 assembler now reuses `integrity::crc16::eac3_crc16` after the complete 4096-byte synthetic frame body has been staged. Its CRC2 rule is bound to `CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1` (manifest SHA-256 `d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6`): init 0, polynomial `0x8005`, MSB-first, xorout 0, generate over `frame[2:4094]`, store big-endian at `frame[4094:4096]`, and require residual 0 over `frame[2:4096]`. The caller no longer supplies CRC2; `D0Crc2IntegrityReceipt` returns the generated U16 and verified residual. Any integrity or layout error precedes the single caller-writer commit. The synthetic vector fixes CRC2 to `e0b0`; D0 assembler tests (3), selected 5.1 frame/CRC regression (1), and 5.1 reconstruction header regression (1) pass. `production_allowed=false`; actual D0 audio, source binding, playback and product output remain open.

## 2026-11-12 synthetic parser references

`FLAT7X-D0-SKIPFLD-C511-PARSER-ADDENDUM-V1` (manifest `cdc12af971b2da49f0a1d21fef783b0d6c195003ce25683c21ad51e4e39a8678`, seal `d2b31ba1427dd470d9b26f84751ab0ca5465ea6605800f559149da8da537ff41`) passes its normal and optimized checker. It records OpenJOC parsing a separately synthesized ID6 C=511 EMDF container as `Parsed`, with 511/511 bytes consumed, and a C=512 pre-frame rejection. Its raw bytes are absent, so local Rust writer tests remain length/copy checks rather than same-byte parser replay.

`FLAT7X-SYNTHETIC-I0-D0-1664-PARSER-V1` (manifest `f949141aee93432e4dc27e126ee03e318dc86d8e2dbc9555c1052e9848e5d09f`, seal `cf28d44ea8e5d6dc6ebe714f4da4f988f33112fcded79d55616a7a1710a51ddf`) passes normal and optimized verification. Its OpenJOC structural reference reports a synthetic I0 2560-byte + D0 4096-byte 6656-byte AU, D0 chanmap `0x1A00`, and block-2 C=511 ID11/ID14/JOC index1/7-channel/15-object parse shape. The package contains expected structure, not frame bytes; it does not parse the local composer output or validate its I0 CRC. Audio-block parse, decode, PCM and real output remain `NOT_RUN`; `production_allowed=false`.

## 2026-11-12 D0 structural integration boundary

The available structural D0 path already composes the approved `chanmape`/U16 `chanmap` BSI pair, typed complete BSI, six-block layout, one complete block-2 skip-field, and generated CRC2 in `assemble_flat7x_d0_frame_bits`. The D0 assembler obtains BSI end from `write_flat7x_d0_bsi_structure`, then writes the skip-field at the layout-computed span with `write_flat7x_d0_block2_skipfld`.

The remaining D0 API gap is the producer boundary: `Flat7xD0FrameBitInput` receives caller-owned opaque block spans, while `D0Block2SkipFieldInput` carries a caller-supplied `CallerOwnedUnboundD0Block2InsertionPoint`. No application/CLI production callsite supplies those values. The next direct Wash contract must bind six D0 block extents and the block-2 insertion cursor to the same source/generation owner; an audio-block producer remains outside the available approved material.

The existing byte composer and elementary-stream composer remain structural components over caller-supplied opaque spans. Wash contracts `FLAT7X-I0-AC3-BSI-CONTINUATION-ETSI-V1` and `FLAT7X-I0-AC3-CRC-GENERATION-ETSI-V1` now bind the I0 frame-part sequence and CRC generation rules. The assembler below uses caller-owned encoded bit spans, generates CRC1/CRC2, and verifies synthetic zero-residual receipts. AC-3 audio-block generation, source-generation joins and product output remain open.

## 2026-11-12 Flat7x I0 AC-3 structural frame assembly

`FLAT7X-I0-AC3-BSI-CONTINUATION-ETSI-V1` (manifest `cddad099b557617ab6f4aa73d3c40d87f75f12a55c141e14bff6573bdc891787`, seal `329d5f9b1a43cdbee31abe4403f398ed4671a9a27e6bec40a2b09679ae46079d`) and `FLAT7X-I0-AC3-CRC-GENERATION-ETSI-V1` (manifest `07ad2b60bd22a1312ff15107cf35e836a12a763c764b3fbaa74e02b4f2d9dc2d`, seal file `14020e164f7805723a0fd90cd77f924961a3a457315f8fdbe4219dafdaccb031`) pass normal and optimized verification. The new `codec::i0_frame_assembler::assemble_flat7x_i0_ac3_frame` composes the approved 62/70-bit header, opaque caller BSI tail, six ordered audio-block spans, optional auxdata and caller `crcrsv` seed into exactly 2560 bytes / 20480 bits. It prevalidates source ranges, extent and capacity, stages the frame, generates CRC1/CRC2, checks zero residuals, then performs one caller-writer commit. Synthetic assembler tests, AC-3 CRC scalar vectors and selected 5.1 regressions pass. Audio generation and source-generation join remain open; `production_allowed=false`.

`codec::ac3_bsi_tail::write_flat7x_i0_ac3_bsi_tail` materializes the BSI fields from the same continuation package. It starts at bit 62/70, emits presence-controlled langcode, audprodie, two timecodes and addbsi, omits acmod=0-only fields, and returns the exact payload extent. Four synthetic integration tests verify field values/order, the 64-byte U6 maximum, unaligned payload placement, invalid fields, cursor mismatch and capacity atomicity. No XML mapping or audio production is added.

`assemble_flat7x_i0_ac3_frame_from_fields` composes that typed BSI tail with six caller-owned opaque audio spans through the existing I0 frame assembler. The all-optionals vector verifies the 70→142 BSI extent; synthetic spans complete the 20,480-bit frame, and CRC residuals plus fail-before-write cases pass. The opaque low-level API remains available; no audio-block producer is added.

The post-commit 5.1 producer audit found no safe typed edge into the I0 span seam. `SelectedMainFrameCandidate` exposes the complete E-AC-3 frame bytes and per-block mantissa bit counts, while `write_frame_body` writes six E-AC-3 block bodies into one shared frame cursor. I0 assembly requires six AC-3 `audblk` bit spans. The selected 5.1 path has no span-producing API, and no approved contract establishes AC-3 reinterpretation of its E-AC-3 block bytes. A separate I0 audio-block producer contract and same-source binding are the typed blockers.

## 2026-11-12 AC-3 first-audblk control prefix

ETSI TS 102 366 V1.4.1 clauses 4.3.3 and 4.4.3.1-4.4.3.13 define the first audblk syntax order through coupling bands: five blksw bits in L/C/R/Ls/Rs order, five dithflag bits, dynrnge with optional 8-bit dynrng, cplstre=1 in block 0 followed by caller-owned cplinu, five chincpl bits, four-bit cplbegf/cplendf, then exactly ncplsubnd-1 cplbndstrc bits. The typed coupling writer requires cplinu=1, at least two selected channels, U4 frequency codes, active subband count 1..=18, and matching boundary-bit length from ncplsubnd=3+cplendf-cplbegf; it serializes caller choices only and stops before cplcoe. Synthetic vectors cover one/three subbands, invalid ranges/shape, exact extents, cursor order and fail-before-write capacity. The mapping is syntax/model input, not a DEE/native behavior claim.

The same-source selected F07/F12 plan has block-0 short flags and dither decisions, while selected BSI/DRC has the block-0 line-dynamic-range code. These can supply the prefix as a user-authorized model projection; E-AC-3 frame bytes are never input. The field inputs carry no source identity receipt. cplinu, chincpl, cplbegf, cplendf and cplbndstrc are explicit caller choices, not selected by the approved I0 target. Coupling coordinates/exponents, BA/SNR controls, mantissa syntax, six-block producer, source-generation join and production remain UNBOUND/NOT_RUN; production_allowed=false.


## 2026-11-12 Flat7x CLI paired-target gate

run_with_data_rate_and_joc_layout now resolves two request routes. ExistingSelectedFivePointOne preserves the single-frame F15/OutputOwner path for all six supported rates. Explicit Flat7x selection resolves the approved I0 640 kbps / 2560-byte plus D0 1024 kbps / 4096-byte target pair (1664 kbps / 6656-byte aggregate), then returns Flat7xPairNotReady before run_worker, input admission or output staging. It cannot fall through to a single 1024-kbps E-AC-3 frame. Targeted route tests pass for pair selection, all six 5.1 rate rows and the pre-I/O fail-closed result. The pair target remains production_allowed=false; I0 audblk production is NOT_RUN, while D0 source-generation binding and pair receipts remain UNBOUND.

## 2026-11-12 one-block AC-3 producer prototype

`codec::ac3_audio_block::produce_ac3_audio_block` now produces one complete block-zero AC-3 audblk from one previous/current 256-sample PCM window per channel. Its explicit surface includes caller-selected cplinu, DRC, block flags, decoder exponents, BAP controls, delta allocation, optional skip data, gainrng and mantissa quantizer policy. The prototype supports acmod 7/lfeon 1, long transforms, chbwcod 48/endmant 217, gainrng 0 and seven LFE bins; `lfeexpstr=1` is required by the block-zero syntax. BAP 1/2/4 groups continue across exponent sets per ETSI 6.3.5, including the full-band-to-LFE boundary.

The validation-only `ac3_audio_block_parser` checks the supported branch, BAP-derived mantissa extents, legal code spaces and exact cursor. Synthetic tests: domain 5 passed; validation parser 4 passed; selected 5.1 frame candidate and prepared-input rate/profile regressions each 1 passed. Formatting, workspace check, rendered-map check and boundary check passed (91/104). Strict workspace Clippy reports nine existing analysis/DRC/JOC lints; no AC-3 source is named. Full workspace tests, W04, real PCM, decoder acceptance, six-block frame output, source-generation receipts and product encoding remain NOT_RUN/UNBOUND. The CLI Flat7x gate remains `Flat7xPairNotReady`; the 5.1 route remains unchanged.

## 2026-11-12 official-demo parameter selection

The active rate override remains I0 640 kbps + D0 1024 kbps = 1664 kbps aggregate (6656 bytes/AU). Other captured profile values follow the approved official-demo supplements: I0 legacy AC-3 BSID6, `acmod=7`, LFE on, `dmixmod=1`; D0 E-AC-3 BSID16, `acmod=5`, LFE off, `chanmap=0x1A00`; six blocks / 1536 samples per frame; JOC spatial index 1, coefficient downmix index 1 with width 7, extension index 0, wire phase signaling off, 15 JOC objects and complexity index type A 16; EMDF IDs 11/14 in group 0. D0's target 4096-byte single frame maps to 11-bit `frmsiz=2047` (`0x7ff`); I0 remains a separate 2560-byte frame. The sample's D0 rate is 640 kbps and remains overridden by the 1024-kbps target.

Wash package `FLAT7X-ID14-OBJECT-COUNT-DOWNMIX-AND-COMPLEXITY-FIELDS-V1` was copied from the sanitized implementer handoff and verified locally in normal and optimized Python modes; manifest SHA-256 `c39d02116ff51e5b653b37df10a5b47c240dbec69b6ad23843f392d4809462dc`, seal SHA-256 `37fcf38657baf35295d5d04531c8ef82d3cbd95ea9f088ec66dcce320edda6cf`. `Flat7xI0D0TargetPlan` keeps spatial and coefficient downmix indices in distinct enums; the target ID14 adapter requires downmix index 1 and semantic object count 15, which the U6 serializer writes as 14. Complexity 16 is a separate target field, not sourced from the native F12 candidate. ADM physical-row count and D0 chanmap remain separate domains.

The official sample does not bind algorithm-level JOC mode/phase controls or the native F12 complexity candidate to this target profile or generation. Those joins remain open. Selecting the measured structure does not assert output acceptance or production readiness; `production_allowed=false` remains in force.

The AC-3 BSID6 wire path carries `dmixmod=1` in the Annex-D `xbs1e` group. The V1 mix-level subset established the four code-4 levels; the later V2 continuation supersedes it for target use by providing the complete observed suffix through `addbsie=0`.

Wash package `FLAT7X-D0-RATE-FRMSIZ-TARGET-V1` passes normal and optimized checks (manifest SHA-256 `1eb4311e3f63ec84c57c7fe636a8a21773bd633ae3f4fe9b0739b88e09e66139`, seal SHA-256 `136c7f4f22687b0508527799ee2a1b783546cdf449d20786bd8a7134894533c9`). The typed target tests now keep the D0 E-AC-3 code 2047 separate from I0 AC-3 `frmsizecod`, frame byte counts, and AU aggregate rate. Native runtime rate-slot/generation joins remain open.

`codec::ac3_audio_block::produce_i0_audio_block_series` adds a production-disabled six-block candidate over one source-bound `SelectedSourceNumericPlan`. It returns all six ordered spans with the same source identity and caller-selected per-block controls; unsupported block-zero exponent reuse and total-budget overflow reject the series as a whole. The F12-spectrum and F09/F10 exponent projections remain explicitly provisional. The current Flat7x source PCM role projection is not yet consumed by this I0 series producer; full-frame assembly, rate fit, source-generation parent join and D0 remain open.

The F15 staging adapter writes one validated pair proposal in per-span order (I0, then D0) and returns its after-owner only after write/sync/readback and CRC checks for both spans. Wrong attempt, offset mismatch or write/readback failure retires and abandons the staging session; the 5.1 append path is unchanged. Its synthetic receipt fixture bypasses F13/F12 producer stages, and the adapter has no product CLI caller or final-file exposure.

## 2026-11-13 official sample I0 BSI continuation

Wash package `OFFICIAL-DEMO-I0-ANNEX-D-BSI-CONTINUATION-V2` passed normal and optimized checkers. Manifest SHA-256 is `85dcd2dfc96bdc57a3caaa7c6affde4eb2be43331afd75bedabf11c62193fa1a`; `package_seal.json` SHA-256 is `a977d2cb4af36a8d1ff44642b15bbf78a90535be94fbdf34866b8126c8ffa716`. It records header and BSI through `addbsie=0`: sample `frmsizecod=36`, `dialnorm=31`, `compre=1`/`compr=1`; Annex-D `xbsi1e=1`, `dmixmod=1`, four mix levels code 4; `xbsi2e=1`, `dsurexmod=1`, `dheadphonmod=0`, `adconvtyp=0`, `xbsi2=0`, `encinfo=0`; and `addbsie=0` with no payload. The sample I0 rate is 640 kbps and the target remains I0 640 + D0 1024 = 1664 kbps.

## 2026-11-13 official sample D0 BSI through chanmap

Wash package `OFFICIAL-DEMO-D0-BSI-THROUGH-CHANMAP-V1` passed normal and optimized checks. Manifest SHA-256 is `35a4527798ccfe6421ce94e380614d721d75a43738bdca0bb9235d2811f2b161`; seal SHA-256 is `5d5902e0bdee4a9a0b8b0e0fe31c975c94ef315fa39f382ecef7cebc1f81c877`. Wash independently recomputed the user-designated MKA SHA and synchronized an 8-file sanitized receipt copy. The first AU is I0 then D0; D0 begins at byte 7,984 and the observed BSI ends at frame bit 76 with `chanmape=1`, `chanmap=0x1A00`.

The observed sample D0 is 640 kbps/2,560 bytes with `frmsiz=1279`. The separate 1,024-kbps target contract remains 4,096 bytes/`frmsiz=2047`. The sample's `dialnorm=31` and `compre=1`/`compr=1` stay sample-specific; this package does not define XML/ADM DRC mapping. The D0 header vector is available for structural serialization; D0 audio generation and complete output remain open.

The local `official_demo_d0_bsi` test replays the sanitized vector through `chanmap` at bit 76 with the existing structural writer. It keeps sample `dialnorm`/`compre`/`compr` as vector inputs and replaces only `frmsiz` with target value 2047. This verifies the captured D0 BSI field sequence and keeps the 640-kbps sample frame separate from the 1,024-kbps target frame; D0 audio-span production remains gated.

`Flat7xI0Ac3FrameHeaderFields::official_demo_sample_non_crc_fields` supplies the observed variable header values while the assembler continues to generate CRC1 from the assembled target frame. `write_flat7x_i0_official_demo_bsi_tail` writes the complete 35-bit suffix from frame bit 70 through addbsie at bit 105. The focused BSI-tail suite passes 7 tests, including all sample field values, exact extent, wrong-cursor rejection, and short-capacity atomicity. The boundary checker replays the V2 package normally and with `python -O`.

The I0 BSI/frame callsite is now materialized as a source-bound frame candidate. It retains the selected `ResolvedEncodingProfile` and pair target, checks the exact `SharedAnalysisIdentity`, then joins the complete official V2 BSI suffix and existing six-block I0 audio-series candidate. The frame assembler generates CRC1/CRC2 from the approved ETSI contract. Auxdata and the initial `crcrsv` bit are explicit caller inputs; both provisional spectrum/exponent assumptions stay attached to the candidate, and `production_allowed=false`. The target remains I0 640 + D0 1024 = 1,664 kbps.

## 2026-11-13 callback source-to-pair input seam

`stage_source_frame` now joins the selected 1664-kbps target, current provisional PCM-row projection, and same-source F12 numeric-plan receipt by `SharedAnalysisIdentity`. The pair-source binding preserves the projection receipt and actual row views for the I0/D0 frame assemblers.

D0 block spans require `Flat7xD0AudioSpanReceipt` with the attempt, source transaction, source index and analysis-after generation. Receipt identity mismatch rejects the pair candidate. The current production callback supplies the typed disabled state `NoApprovedAudioProductionRule` and exits before the selected single-frame output staging seam. `Provided` spans can currently be constructed only by module tests; the I0/D0 audio-block candidate helper is ready for an approved producer and explicit I0 caller policy.

Validation from the preceding increment: formatting, scoped workspace check, the two D0 receipt boundary tests, the selected 5.1 all-rate route regression, and boundary inventory passed (`91` scoped tasks, `104` IDs, `0` unmaterialized roots, `6` partial roots). The remaining first-output blockers include a Wash-approved D0 audio-span producer with a same-source receipt, I0 audio-assumption evidence, pair F13/I0/D0 integrity receipts, product output, and W04.

The new I0 frame-candidate increment composes the official V2 suffix and the source-bound six-block series through the existing 2,560-byte assembler. The candidate retains the `SharedAnalysisIdentity`, pair role-map receipt, both provisional audio-assumption IDs, and generated CRC receipts. It requires caller-supplied auxdata and `crcrsv` seed; the structural vector supplies synthetic auxdata to complete the fixed frame extent, and the product auxdata policy remains open. The candidate remains `production_allowed=false` and does not enter AU publication.

Verification: source-bound series/frame test 1 passed; I0 frame-assembler and BSI-tail suites 20 passed; pair-source tests 2 passed; Flat7x target tests 11 passed; the selected 5.1 all-rate route regression 1 passed. Scoped workspace check and formatting passed. Map rendering, supplement catalog, and boundary checks passed (`91` verified scoped tasks, `104` IDs, `0` unmaterialized roots, `6` partial roots). Clippy completed with 0 errors and 69 warnings. No DEE/encoder or raw-media access occurred.

## 2026-11-13 source-bound I0/D0 in-memory AU candidate

`assemble_flat7x_pair_access_unit_candidate` now takes the source-bound six-span pair candidate, the complete I0 frame candidate, and an explicit typed D0 BSI structure. It verifies the selected `ResolvedEncodingProfile`, role-map receipt, `SharedAnalysisIdentity`, and D0 attempt/transaction/callback/source/generation fields before assembling D0 and composing I0→D0 bytes. The D0 assembler validates six ordered opaque spans, one nonfragmented block-index-2 skipfld, C=1..511, the 4,096-byte target and CRC2; the AU composer validates I0=2,560 bytes, D0=4,096 bytes, and 6,656-byte order. The returned pair candidate retains target/profile, both frame receipts and bytes, source identity, and remains `production_allowed=false`; it does not call CLI, AU owner, file, GTC, or publication.

The pair tests use synthetic caller-owned spans and BSI values: C=511 produces the candidate; C=512, a one-bit-short D0 frame, and mismatched source identity reject it. The test verifies block index 2 is the third prefix, the AU layout is I0 at bytes 0..2560 followed by D0 at 2560..6656, and the D0 CRC2 residual is zero. A separate writer regression consumes the sealed three-AU numeric vector C=314/301/301 using synthetic container bytes; the sample is 640 kbps and is not used to infer the 1,024-kbps target's C distribution.

Wash package `DEE67-PROENCODER-SOURCE-RANGE-METHODSET-BOUNDARY-V1` was cataloged and checked normal/optimized. It observes selected same-object worker range reads without direct stores; the external range populator and D0 audio owner remain `UNBOUND`, so D0 spans stay caller-owned at this candidate seam.

Pair candidate verification: pair/domain tests 7 passed; sample numeric writer test 1 passed; existing D0 frame/AU composer suites 6 passed; application pair-source tests 3 passed; selected 5.1 all-rate regression 1 passed. The next blocker is a source-generation-bound D0 audio/EMDF producer with a target-valid BSI/cursor receipt; the candidate remains out of CLI and file publication.

## 2026-11-13 target clarification and Wash boundaries

The target is 1,664 kbps aggregate, with the official-demo non-rate profile and only the rate changed: I0 640 kbps/2,560 bytes plus D0 1,024 kbps/4,096 bytes. The modeled profile retains 48 kHz, six blocks/1,536 samples, Flat7x spatial index 1, coefficient downmix index 1, D0 chanmap `0x1A00`, and 15 JOC objects. The 1,280-kbps reference sample supplies the non-rate values; its D0 C distribution is not projected onto the 1,024-kbps target.

Wash package `DEE67-PROENCODER-SLOT0-RANGE-WRITE-BOUNDARY-V1` (manifest `bb49bd95f41c7eee237299ce383075d00943343c0c7899a08b21a40db41574bd`, seal `912b38d37f98625ee3a039c08c7902d35ec27ba742ca5e0b0634d79735496b62`) and `DEE67-PROENCODER-SLOT0-CONFIG-READ-BOUNDARY-V1` (manifest `bf2f7e43a4cecd315e21d993c7fddcd8534089224747db1d9d2fc805f914e95d`, seal `2d906abfdb78fe2ca17ed21e1a0aeb3075e95181dfedd6209a6957d3866033d9`) pass normal and optimized checks. Both are slot-0-only observations; external D0 span population and the profile-index-1 consumer remain `UNBOUND`.

Wash package `DEE71-I0-D0-AU-COMMIT-BOUNDARY-WASH-V1` (manifest `ba65817c81f8097dda3cc3897f337da81c9a46fee0a892051d553c49089e8f04`, seal `92ffa1712763fccffd6006f8d9b2b35b7cc4b7660bef304b28b1bb0f3bc68dfa`) passes normal and optimized checks. It records separate callback-buffer and FileWriter paths without a same-source I0/D0 generation join; the current candidate stays in memory and production-disabled.

## 2026-11-13 application wrapper integration test

`application_pair_candidate_composes_matching_i0_d0_frames_in_one_au` passes with the opt-in `test-support` feature. It runs the application wrapper over one matching `SharedAnalysisIdentity`/role-map receipt, a source-bound I0 frame candidate, six synthetic caller-owned D0 spans, and explicit D0 BSI. Assertions cover 6,656-byte AU size, I0 at bytes 0..2560, D0 at bytes 2560..6656, exact 4,096-byte D0 extent, one block-index-2 skip-field, and `production_allowed=false`. The synthetic receipt helper is feature-gated and absent from the default production build.

Commands and results:

- `cargo test -p jocforge-application --lib application_pair_candidate_composes_matching_i0_d0_frames_in_one_au --features test-support --locked -j1` — 1 passed.
- `cargo test -p jocforge-application --lib flat7x_pair_source --locked -j1` — 3 passed.
- `cargo test -p jocforge --lib selected_five_point_one_keeps_every_supported_rate_on_its_single_frame_route --locked -j1` — 1 passed; `origin/experimental/independent-encoder` was at `514442cade7246d5b7108ef999a1f42c448f8944` when run.

The next implementation edge remains the real source-generation-bound D0 span producer and the durable I0/D0 pair commit. This application test validates the in-memory seam only.

## 2026-11-13 application-to-output staging gate

`preflight_flat7x_pair_io_stage` validates a completed application candidate against the expected `SharedAnalysisIdentity`, the role-map/D0 identity join, the I0/D0 extents, and the I0-before-D0 AU spans. It then exercises the existing `Flat7xAuOutputOwner` in memory with absent F13/I0/D0 receipts. The expected result is a `StageRejected(F13MetadataReceiptMissing)` retirement with unchanged committed owner state; the receipt records no append proposal and no I/O attempt. The candidate remains `production_allowed=false`, and the CLI route remains `Flat7xPairNotReady`.

Verification: the application pair integration test passed (1), the application pair-source tests passed (3), and the selected 5.1 all-rate regression passed (1). `cargo check -p jocforge-domain -p jocforge-application --locked -j1`, fmt, scoped Clippy, map rendering and boundary check passed. No file, DEE run or encoder run was produced.

The typed `Flat7xAuAppendProposal` I/O boundary now also has a test-only `Cursor<Vec<u8>>` sink. A fully source-receipted synthetic pair is copied and read back through the shared extent/CRC path; the sink supplies no fsync or durable claim, so F15 rejects completion with `AppendReceiptCandidateMismatch` and preserves the owner state. No staging file is opened.

## 2026-11-13 ProgrammeOutputStaging pair seam

`ProgrammeOutputStaging::append_flat7x_au` is a crate-local, uncalled bridge from a domain-issued F15 pair proposal to `AtomicOutputStaging::append_flat7x_au`. It checks the programme and attempt binding, paired I0/D0 source identity, F13 transaction, target frame/AU extents, AU offset, and CRC receipts before staging opens. The production CLI still returns `Flat7xPairNotReady`; the seam does not enable `production_allowed` or add a caller.

Three focused unit tests cover attempt mismatch, programme mismatch, and matched pair delivery to an in-memory cursor followed by nondurable F15 rejection. The receipt handoff integration and I/O memory-sink tests pass; the selected 5.1 route regression also passes. Format, workspace check, generated-map check, and boundary check pass (92 scoped tasks, 104 IDs, 0 unmaterialized roots, 6 partial roots). Strict Clippy reports existing diagnostics in domain and CLI code, with none pointing to the new pair staging checks. No staging file, `.ec3`, encoder, or DEE run occurred.

## 2026-11-13 CLI-to-D0 route progress

The Flat7x selector now enters ADM/job admission and the real source-stage pipeline with the 1,024-kbps D0 rate row. It reaches the first source's explicit `NoApprovedAudioProductionRule` input and returns the typed `Flat7xD0AudioProducerUnavailable` error before pair/output staging. The selected 5.1 route and all six supported rate rows remain unchanged.

The source boundary carries a provisional seven-carrier PCM view: I0 front L/C/R plus the four mapped D0 chanmap rows. The row selection is a model-owned assumption and does no audio encoding. A typed `Flat7xD0AudioSpanReceipt::from_source_bound_producer` is available to a future producer. The pair proposal API consumes a source-bound I0 frame, caller-owned opaque D0 frame bytes, an explicit incrementable JOCForge generation label, and same-source F13/I0/D0 receipts; it composes the 6,656-byte I0→D0 AU and stages it through the existing atomic sink.

The remaining audio-core gap is a D0 seven-carrier E-AC-3 block writer. The local I0 block producer is fixed at five full-band rows plus LFE, while the selected D0 row uses `acmod=5`, `lfeon=0`, `chanmap=0x1A00`. No D0 audio, BSI, or EMDF bytes are generated or inferred in this increment. A typed seven-carrier audio-block producer and its F12 integrity receipt are the shortest next seam.

## 2026-11-13 source route and provisional D0 row view

The Flat7x/1024 CLI request now enters the existing ADM/job worker and source callback flow. On the first source, the current source stage returns the named `Flat7xD0AudioProducerUnavailable` error with `NoApprovedAudioProductionRule`; no pair AU or output file is staged.

The source binding carries `Flat7xD0CarrierPcmRows` under the replaceable model `PROVISIONAL_D0_FRONT3_PLUS_CHANMAP4_V1` (I0 front L/C/R followed by four D0 chanmap rows). It is a borrowed row view with extent checks. Pair generation is a separate explicit `Flat7xSourceGeneration` input, incrementable through `checked_next` and independent of DEE or analysis-generation values.

The pair proposal seam accepts the source-bound I0 full-frame candidate, opaque caller-owned D0 frame bytes, explicit generation label, and same-source F13/I0/D0 receipts; the existing F15 owner checks the pair CRCs, extents, skip-field bounds, and I0→D0 order. `ProgrammeOutputStaging::append_flat7x_pair` delegates the resulting proposal to the existing atomic sink. The CLI route has not yet been given a D0 producer.

The previous seven-row D0 PCM view mixed three I0 front channels with four D0 chanmap rows. The Wash-approved profile row specifies four D0 coded channels (`Ls`, `Rs`, `Lrs`, `Rrs`); its seven JOC reconstruction carriers are a separate domain. The shortest remaining audio implementation seam is a four-channel, no-LFE D0 audblk writer and its source-bound span receipt. Existing `produce_i0_audio_block_series` implements only five full-band rows plus LFE. No D0 audio/BSI/EMDF bytes were generated. The 5.1 six-rate route remains unchanged.

## 2026-11-13 D0 coded-channel shape correction

Wash-approved package `JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1`, `PROFILE-ROW.json`, defines D0 coded-channel locations `[Ls, Rs, Lrs, Rrs]` with `acmod=5`, `lfeon=false`, and `chanmap=0x1A00`. It separately defines seven JOC reconstruction carriers `[L, R, C, Ls, Rs, Lrs, Rrs]`. The previous `PROVISIONAL_D0_FRONT3_PLUS_CHANMAP4_V1` model is superseded because it mixed the two domains.

The local source binding now carries the four D0 PCM rows already produced by the provisional source-role projection. That DirectSpeakers-to-D0 projection remains a replaceable assumption. The four-channel D0 audio-block producer, receipt issuance, atomic pair-root commit, final output exposure, and actual encoding remain open; the 5.1 path remains unchanged.

The D0 increment now includes `codec::exponent_packing::write_eac3_exponent_row`, registered against `EAC3-EXPONENT-U4-U7-WRITER-BOUNDARY-V1` (manifest SHA-256 `d1fb9e452554c04bb6c630d143e8124a8790d8610ac20deb61838d14e2f7a2e9`). It preflights `4+7*N` bits and writes caller-owned U4/U7 values with a cursor receipt. It does not select seed/group values or bind a D0 lane. The complete four-channel D0 audblk, BAP/mantissa production, six-block history, pair-root commit and actual output remain open.

## 2026-11-14 typed D0 frame-tail policy

The Wash-sealed `FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1` (manifest `689d5e4f3b27fd79110f6fb99189d4407384424929b3a74d20ba3447698f545b`, seal `5f0c525745784e7b41714d04e5dc432fc2689ccf6ae74e925afe3cf7662e82b6`) now has a typed implementation seam. `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits` travels on `Flat7xD0AudioSpanReceipt` through pair staging to the AU/frame composer. The writer emits zero-filled `nauxbits`, followed by `auxdatae=0` and `crcrsv=0`, then final CRC2. The receipt exposes the `nauxbits` span and both terminal values. `encinfo` belongs to the BSI extension. The assembler enforces one complete block-2 skip-field and existing 1–511 byte bounds. Arbitrary opaque tail input is gated by `test-support` and retained only for a synthetic structure-copy test.

Focused verification passed: 5 default D0 frame tests, 7 frame tests with `test-support`, 1 application pair integration, 1 selected 5.1 all-rate regression, and domain/application check. Generated-map and boundary checks pass (92 scopes / 104 IDs / 0 unmaterialized roots / 6 partial roots). Strict Clippy still fails on 11 existing diagnostics outside the changed D0 seam. The small frame-tail increment does not create D0 audio bytes or connect the CLI producer. Next is D0 four-lane/253-bin BAP and six-block audio production, then same-source F12/F13/F15 admission and parent commit. Fresh GO remains required before encoder execution or `.ec3` creation.

## 2026-11-14 D0 BAP provider and six-block rows

The local `codec::bap::flat7x_d0_provider` now materializes an isolated D0 table set from ETSI TS 102 366 V1.4.1 Tables 6.6–6.12 and 6.14–6.16, identified by canonical SHA-256 `5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe`. Its typed API allocates a 253-bin row or six blocks × four D0 coded rows; no LFE axis exists. Existing selected 5.1 BAP provider/allocator stays untouched.

Verification: table landmark/coverage 1, D0 provider/frame 4, existing 5.1 BAP 14, workspace check, and selected 5.1 all-rate regression 1 passed. Generated-map and boundary checks pass (92 scopes / 104 IDs / 0 unmaterialized roots / 6 partial roots). Strict domain Clippy reports 9 existing diagnostics outside this D0 provider. This is a BAP-row allocator only: transform, D15 exponents, frame-wide candidate/budget selection, mantissas, audblk serialization, source-bound receipt, CLI wiring, parent commit and real output remain open. No encoder or `.ec3` was run.

## 2026-11-14 D0 spectrum rows and gainrng

`codec::flat7x_d0_spectrum::transform_flat7x_d0_pcm_block` now accepts one typed 4-channel D0 block with previous/current 256-sample windows. It calls the shared pure `transform::long_mdct::forward_long_mdct_512` core and returns 253-bin coefficient words plus measured `gainrng` codes using Table 6.33a. D0 source history remains caller-owned; this pure candidate creates no state commit and no audio-block syntax.

The D0 spectrum suite passed (2), BAP suite (4), existing selected 5.1 BAP suite (14), and 5.1 all-rate CLI regression (1); domain check passed. The next small seam is D15 exponent extraction and U4/U7 row production from these spectra, followed by mantissa packing and six-block audblk assembly.

## 2026-11-14 D0 D15 exponent row

The D0 row stage extracts raw exponents from 253-bin spectra, applies D15 `JOCFORGE_D15_LOWER_ONLY_SEED15_SLEW2_V1`, returns the decoder-visible exponents and serializes a U4 seed plus 84 U7 groups through the existing writer (592 bits per row). The preprocessor caps the seed at 15, forward-limits increases to two, then backward-lowers earlier exponents to bound decreases. This keeps decoder exponents no larger than raw and every delta within ±2. Wash confirmed ETSI does not select one unique preprocessor and recommended this lower-only policy. It remains an explicitly replaceable JOCForge model choice.

The focused D15 suite passed (6), including the four raw-exponent spike cases, 10,000 deterministic invariant rows and U4/U7 byte serialization; the 5.1 all-rate regression remains green. Mantissa normalization using the final decoder exponents, whole-frame bit budgeting, complete D0 audblk, source receipts and parent commit remain open. No encoder or `.ec3` was run.

## 2026-11-14 D0 mantissa block stage

`codec::flat7x_d0_mantissa::pack_flat7x_d0_mantissa_block` now joins one typed spectrum block, four matching D15 decoder-visible exponent rows, and one selected block from the six-block D0 BAP frame. It quantizes the 4×253 coded-channel coefficients through the existing E-AC-3 quantizer and grouped packer, preserving `[Ls, Rs, Lrs, Rrs]`; the returned candidate includes item observations, grouped fields, MSB-first bytes and bit count. The named local choice is `JOCFORGE_EXISTING_EAC3_QUANTIZER_V1`; production remains disabled.

Two synthetic input-only tests passed. One verifies every item against the shared quantizer using the final decoded exponent and expected D0 lane order; the other verifies selected block BAP rows, grouped fields and output extent. Wash's quantizer package has no expected code vectors, so these are adapter/packing checks, not independent conformance results. Frame-wide BAP/bit-budget selection, full six-block D0 `audblk` production, source-history owner and F12/F13 receipts, CLI caller, and parent F15 atomic commit remain open. Continue with complete `audblk` assembly and its budget before wiring the source producer; actual encoding remains Fresh-GO gated.


## 2026-11-14 D0 audblk and full-frame candidate

The isolated codec writes a typed 23-bit D0 `audfrm` prefix and six D0 `audblk` spans from four coded channels. Its frame candidate accepts six caller-provided spectra and D15 rows, BSI fields, optional dynamic-range codes, and one complete EMDF container. It searches raw q values 1023 down to 1, measuring exact BAP-grouped mantissa cost for each full six-block frame; no monotonic cost assumption is used. It selects the fitting candidate with the maximum encoded content bits, with higher q as the deterministic tie-break. q=0 stays a valid zero-BAP input vector and is excluded from playable candidates.

The selected candidate then passes through the frame assembler, including the block-two skip-field, zero-filled aux-tail and CRC2. Three synthetic D0 frame tests, five frame assembler tests, four six-block layout tests, eleven D0 unit tests, four application pair-source tests and the selected 5.1 all-rate regression passed. This candidate has no callback PCM history, source-role receipt, F12/F13/F15 receipts, CLI callsite or parent commit; it remains `production_allowed=false`.

The next critical slice is to produce these six spectra from callback PCM under a programme/attempt/source-generation history owner, then bind same-source DRC and F13 receipts, D0 F12 integrity, CLI routing and parent F15 atomic publication. Real-input acceptance and W04 follow after Wash Fresh GO.

## 2026-11-14 D0 source PCM block-series candidate

Added `codec::flat7x_d0_pcm_source::stage_flat7x_d0_pcm_block_series` and `produce_flat7x_d0_frame_from_pcm_source`. The source stage consumes four projected rows in `[Ls, Rs, Lrs, Rrs]` order and a caller-supplied 256-sample overlap value; builds six consecutive 512-sample windows, six 253-bin spectra and four D15 rows per block; then the new frame entry passes them into the existing D0 BAP/q-search/audblk/frame composer with caller-owned BSI, EMDF, BAP parameters and dynamic-range codes. The returned complete frame candidate keeps the last 256 PCM samples alongside it as proposed history.

The three PCM-series unit tests, four D0 frame integration tests, and workspace `cargo check --locked` passed. Strict domain Clippy still reports 9 pre-existing diagnostics in analysis, DRC and JOC modules; none point to this slice. Full package test linking hit stale Windows PDB failures in unrelated integration-test binaries; the directly scoped D0 frame suite linked and passed.

The application seam `produce_flat7x_pair_d0_pcm_frame_candidate` now takes the exact four D0 rows from `Flat7xPairSourceBinding` and keeps the binding's D0 source identity and role-map receipt with the frame/history candidate. Its focused application test passed (1).

Still open: install history in a programme/attempt/source-generation owner; resolve history over suppressed source indices 0/1 under the F15-only commit rule; carry valid/padded counts; issue same-source I0/D0/F13/F12 receipts; and connect the CLI pair caller, parent commit and durable output. No production caller or file output was added. Fresh Wash GO remains required for an encoder run.

## 2026-11-14 role-specific F12 pair receipt proposal

Added `Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024` and the noncommitting `IntegrityProgrammeOwner::stage_flat7x_i0_d0_f12_pair_for_callback`. One pair proposal validates I0 at 640 kbps / 2,560 bytes with `AbsentPublishedPayloadV1`, validates D0 at 1,024 kbps / 4,096 bytes with one protected block-two EMDF at the supplied bit cursor, and returns two F12 receipt sets with the same programme/attempt/transaction/source/dispatch/AU identity, before generation, after generation and one owner receipt. The committed Integrity root remains unchanged until a parent installs the after-root.

The focused pair test passed (1): it checks both role-specific frame sizes, the I0 no-payload cursor, the D0 protected container position, CRC residual zero for each frame, one shared generation step, and failure atomicity for a wrong D0 container cursor. Existing 5.1 scalar integrity staging is unchanged. Still open are actual F13/source identity joining, source-generation and valid/padded extents, application/CLI pair staging, and F15 durable append installation. `production_allowed=false`.

## 2026-11-14 Wash source-map role and AU-shape refresh

Registered `JOCFORGE-MAIN-ISOLATED-7.1-SOURCE-MAP-WASH-V1` (manifest `1c66063cd651daa8c7b0fc1b205db4d9521a54ca33b645e711ebd13989a3dd58`, seal `e799eebf51702a863212fb2f5a27b9ad2c773768e384ecdc2b95cb844f8f4d81`). The package-only checker passed in normal and optimized Python; all 18 isolated snapshot file hashes match the recorded HEAD. This implementation mapping consumes only typed row/carrier role separation and explicit I0→D0 AU order; legacy audio/rate paths remain excluded as implementation inputs. The package keeps `production_allowed=false` and reports the CLI pair route uncalled.

## 2026-11-14 source-bound pair F15 append candidate

Added `stage_flat7x_pcm_pair_for_f15`: it joins the source-bound I0 and D0 frame candidates, the role-specific F12 pair proposal, F13 metadata and an explicit `Flat7xSourceGeneration` into one ordered in-memory F15 AU append proposal. The candidate carries D0 `after_history` and access to the matching Integrity after-root/owner receipt. Neither state is installed until F15 reports durable append success.

The focused application integration test passed (1), using the test-only receipt-gate metadata fixture and generated I0/D0 frame candidates. It verifies the 6,656-byte I0→D0 sequence, one F13 identity, shared F12 pair identity, and retained D0 history. No file I/O or owner installation occurs here; production and valid/padded source extents remain open.

## 2026-11-14 source-bound F13/F14K pair-integrity bridge

Added `application::flat7x_pair_integrity::stage_flat7x_pair_f12_integrity` to join the D0 source identity/role-map receipt, I0 frame identity, F13 MetadataSourceProposal, F14K protected container and the role-aware rate plan. It derives the D0 payload cursor from the frame assembly receipt, verifies metadata consumer transaction/source/dispatch/AU identity, and returns the noncommitting pair F12 proposal.

The focused pair test exercises source binding → F13/F14K → I0/D0 F12 receipts → F15 in-memory 6,656-byte AU candidate and passed (1). Its F13 metadata is the sealed input-only test fixture. Production callback wiring, source valid/padded extent, durable sink append and one parent state/history install remain open.

## 2026-11-14 reader-bound PCM extent propagation

Normal callback input now forms a `Flat7xD0PcmSourceExtent` from the reader's exact first sample and valid/padded sample counts. The value propagates unchanged through source rows, the Flat7x projection and pair binding, then must match the D0 frame producer settings. The constructor checks callback→source mapping, `first_sample = callback × 1536`, and total extent; the D0 source core checks zero-valued padded tails.

Cached-drain callbacks currently reuse cached PCM rows and carry no source-specific extent, so the Flat7x pair source stage stops at that boundary. This avoids assigning one cached sample interval to multiple codec sources while preserving the existing 5.1 path. The normal source path has a concrete typed extent; D0 history ownership, terminal padding policy, production pair settings/receipts and the F15 parent commit remain to be connected.

Verification completed without running test binaries: formatting, workspace production check, domain/application/CLI test-target compilation, generated-map check, and boundary check passed (92 scoped tasks / 104 IDs / 0 unmaterialized roots / 6 partial roots). Workspace Clippy reports 18 existing errors and 1 warning; no diagnostic points to the files changed in this slice. No `.ec3` or DEE run occurred.


## 2026-11-14 Flat7x D0 history and production candidate

Flat7x `CodecOwnerRoot` now carries optional D0 overlap history as 4×256 F32 bit words in canonical state. The zeroed cold and programme reset values are explicit; the selected 5.1 root remains in the prior schema. Suppressed sources 0/1 commit their D0 PCM block-series history through the existing callback transaction. Cached drains use the replaceable `CachedDrainRepeatPreviousCallbackV1` policy to repeat the prior cached PCM rows.

For source >=2 the production source stage now invokes the complete D0 PCM-to-frame candidate with same-source reader extent, Codec history and F14K container. It also builds a provisional I0 frame, paired F12 receipts from actual F13/F14K, and a 6,656-byte F15 in-memory candidate under named replaceable policies. It stops at `Flat7xPairOutputNotConnected` because the pair-aware durable output owner, multi-AU state, parent root installation and final exposure remain to be connected.

Validation completed: workspace `cargo check --locked`, CLI test-target compile check, fmt, generated-map check and boundary check passed. Strict Clippy reports 18 existing errors and 1 warning outside this slice. No test binary or encoder ran. The next slice should build the same-source I0 frame and paired F12 proposal, then connect F15 and the existing durable sink.

## 2026-11-14 aggregate 1664-kbps F15 output profile

F15 admission now receives the CLI-resolved Flat7x layout. It creates a 1,664-kbps / 6,656-byte aggregate output configuration, and the READY root checks the same extent. The codec carrier profile remains D0 1,024 kbps; the selected 5.1 output rows remain unchanged.

The source callback builds a complete in-memory I0/D0/F12/F15 candidate using named local policies, then stops before durable output-owner persistence. The first-AU pair owner is still cold per candidate; multi-AU state, atomic output append, parent root swap, and EOF finalization are the next connections. No output file was written. Wash reports the 5.1 all-rate regression passed at exact commit ad6f2e77; the current output-profile changes need their own exact-commit regression before any compatibility claim.

## 2026-11-14 pair candidate and aggregate output configuration

The production callback now builds an actual source-bound I0 candidate from the CLI's provisional six-block policy, pairs it with the source-bound D0 PCM frame, stages actual F13/F14K through the role-aware F12 pair path, and creates an in-memory 6,656-byte F15 append proposal. The F15 generation value is the named local model JOCFORGE_FLAT7X_SOURCE_GENERATION_SOURCE_INDEX_V1; the I0 BAP controls and aux tail are likewise replaceable policies.

The output plan now receives the resolved Flat7x layout. It projects a 1,664-kbps/6,656-byte aggregate output owner, and the selected 5.1 route keeps its original per-frame output size. The pair proposal currently stops before durable output-owner persistence and the single parent swap; no candidate file was written.

Verification: fmt, workspace check, CLI test-target compilation, generated-map check and boundary check passed (91 locally verified scopes / 104 inventoried IDs / 0 unmaterialized roots / 6 partial roots). Clippy remains at 18 pre-existing errors and 1 warning outside the changed files. No test binary/encoder ran. Wash reports the 5.1 all-rate regression passed at prior commit ad6f2e77; current pending output-profile changes still need an exact-commit regression before claiming compatibility.

## 2026-11-14 full-input Release candidate and Wash inspection

The D0 frame-tail correction reports zero-filled `nauxbits`, `auxdatae=0`, and `crcrsv=0`. Focused D0 assembly tests pass 5/5; selected 5.1 all-rate regression passes 1/1; corrected-source Release build passes.

Release attempt `1794654566716793500` produced `D:\JOCForge-attempts\love-song-flat7x-1664-release-20261114-190926\love-song-flat7x-1664.candidate.ec3`: 8,417 AUs × 6,656 bytes, 56,023,552 bytes total. Wash's OpenJOC syntax-only inspection found I0 640 kbps / 2,560 bytes and D0 1,024 kbps / 4,096 bytes in each AU, correct 48-kHz / six-block / 7.1 structure, and `malformed_aus=0`. JOC remains absent: zero EMDF containers, zero addbsi-signaled AUs, and 50,497 unresolved skip fields; AU0 is partial with no profile/object/complexity information. Render and playback remain `NOT_RUN`; the candidate is not DEE-equivalent and `production_allowed=false`.

Fix commit `6f3cb91e39d9a8fd308b2ed5f0188d7f67babd10` was pushed before corrected Release attempt `1794658627622854200`. It captured 8,417 AUs / 56,023,552 bytes, with 6,656 bytes/AU, CRC residual 0, and a 288-byte protected F13 container on the first published source. OpenJOC 0.20.1 review is pending. The candidate remains experimental and not DEE-equivalent.

## 2026-11-14 D0 audfrm strategy-field placement

OpenJOC 0.20.1 reported `skip_observed=0` for the previous candidate. ETSI TS 102 366 V1.4.1 Annex E.1.2.3 places six coupling-state bits and 24 two-bit `chexpstr` values in `audfrm`, after `skipflde` and before any `audblk`. The writer now puts these 54 bits in that frame-level span and removes the duplicated per-block fields. Follow-up syntax review also corrected D0 `audblk` ordering per Annex E.1.2.4: four `dithflag` bits precede the `dynrnge` presence/value. Total frame extent stays 4,096 bytes; block-2 skip-field and frame-tail policies are unchanged. D0 audio-frame 4/4, frame assembly 5/5, frame-control 2/2, selected 5.1 all-rate 1/1, `cargo check -p jocforge --locked`, and fmt check pass. Short diagnostic attempt `1794661230282761900` captured 137 AUs / 911,872 bytes. AU0 receipt: audfrm `[94,171)`, skipflde 104, skiple 12,743, skipl 12,744 / C=230, EMDF `[12,753,14,593)`, nauxbits `[32,494,32,750)`, auxdatae 32,750, crcrsv 32,751, CRC2 `[32,752,32,768)`. OpenJOC 0.20.1 still reports `skip_observed=0`, 822 unresolved skip fields, zero EMDF/JOC, `malformed_aus=0`; first parser boundary divergence remains open. `D:\testADM.wav` is diagnostic only; Love Song remains the full-run target.

## 2026-11-15 D0 CI16 addbsi and parser boundary

Wash package `official_demo_d0_addbsi_ci16_v1` (manifest `2a15393cecf15a5f03e58e8f788ff0bd9ea9cf50456b7b23a0ca06f0744112dc`, seal `f1d4da8420911e723098d6f7b983226851928f069ada1cbec18eae87f7607b9b`) supplies the official sample's D0 suffix: `mixmdate=0`, `infomdate=0`, `addbsie=1`, `addbsil=1`, and two CI16 bytes `0x01 0x10` at bits `[85,101)`. Raw bytes were not persisted. Parser-side shape is extension-present in byte0 bit0, zero reserved bits in byte0 bits1..7, and U8 complexity index 16 in byte1; its join to Type-A `complexity16` remains `UNBOUND`.

The approved Flat7x target carries these two bytes, bind_flat7x_d0_bsi_source_inputs passes them through Flat7xD0BsiSourceInputs, and the typed D0 BSI structure requires them. The writer emits both bytes with addbsie/addbsil. The official-profile caller selects mixing=None, yielding mixmdate=0 and infomdate=0; the lower-level mixing writer remains available to another explicit profile. OpenJOC's parser-diagnostic supplement (manifest 74d14c065df0da40772051b7f87e5c91dd0e0190cc2b60f98a73a6923c322b45, seal 163432856af5ba942ce1d37efb0db39f5350cf76c5b02ef82ebb1bf0f4a25663) records parser-only clues for BSI admission and the Huffman truncation boundary; executable/source build join remains UNBOUND.

Focused verification passed: D0 BSI structure 6/6, frame assembler 5/5, frame layout 4/4, audio frame 4/4, application Flat7x tests 6/6, selected 5.1 all-rate route 1/1, CLI check, and fmt check. No new encoding attempt was started. The previous short diagnostic candidate parsed skip field and EMDF on 137/137 AUs, while its ID14 payload remained malformed with truncated Huffman / missing addbsi reports; it predates this CI16 connection. The current target remains experimental with `production_allowed=false`.

Wash additionally reports that OpenJOC 0.20.1's three direct JOC parser calls pass `codeword_bits=None`; the observed `TruncatedCodeword` therefore comes from physical bit-reader EOF. Supplement `OPENJOC-E944-HUFFMAN-CODEWORD-LIMIT-SOURCE-WASH-V1` (manifest `57a4191bf8142d3274e8fa969fcebe22ca2f1316099e4e5b6b86ca30470df3bd`, seal `d9faa26c5bb992680415289772413af5c99698cf54baf0d7d697a32f497c2eb5`) is reported normal/O checked. The OpenJOC executable/source build join remains `UNBOUND`. The next diagnostic boundary is the presence and extent of JOC payload bits after D0 addbsi admission.

## 2026-11-15 E-952 D0 CI16 short diagnostic

Wash Fresh GO E-952 authorized one Release run with source HEAD `2e5c38055a17c3828eb592e6faf2100952dd5927`, binary SHA-256 `93ad79d1554ea4aaca891d45913cc20f59eca0379ad9d94f3b68823e44fda201`, input `D:\testADM.wav` SHA-256 `44434c6002bea5affe5dd640f15aee6efe692905909725d81dc97930f8c098ab`, OpenJOC 0.20.1 SHA-256 `2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39`, and runner SHA-256 `55effa5d59791b11151b1440a5370d9f48eeb7959fda8ee759e971b903eb86b6`. GO marker SHA-256 was `b49bdb423732370d4d812ef88310d890c38c86ad28297d1690e2ab1f5f16739f`.

Attempt `1794675000832477100` naturally completed: 137 AUs / 911,872 bytes, program-reported candidate SHA-256 `ee773d005a74ba7494cfbf3715738c03541ae3cf1243cc039fc33ab3594e84b2`. Run receipt: `D:\JOCForge-attempts\logs\test-adm-flat7x-ci16-bsi-20261115-004427.run.txt`; candidate: `D:\JOCForge-attempts\test-adm-flat7x-ci16-official-bsi-20261115-004427\output\test-adm-flat7x-1664-ci16-bsi.candidate.ec3`; OpenJOC JSON: `D:\JOCForge-attempts\logs\test-adm-flat7x-ci16-bsi-20261115-004427.openjoc.json`.

OpenJOC reports correct outer I0+D0 / 48-kHz / six-block / 7.1 / `chanmap=0x1A00` structure, 137/137 skip fields parsed, 137 EMDF containers, and IDs 11/14 present. CI16 admission is visible on all AUs (`addbsi_signaled_aus=137`, `complexity_indices=[16]`); the prior missing-addbsi diagnostic is gone. ID14 still parses 0/137: first failure is `TruncatedCodeword` from physical reader EOF; `malformed_aus=137`, stream parse fails, decoder admission is false. Deployed compatibility passes; ETSI strict reports payload flag deviations. Decode, render, and playback are `NOT_RUN`; raw payload was not read, `production_allowed=false`.

The coefficient-body/profile join remains UNBOUND. Selected 5.1 uses ID14 selector 3 with five rows; Flat7x keeps payload selector 1 separate from header reconstruction index 1. Wash package official-demo-au0-d0-skipfld-id14-block-ordinal-wash-v1 (manifest 9e42b53bbf3ae5d01f7161af079958adf2c9d26b7ee33a7bcb36a7e3cb572958) reports D0 block 2, channel_count 7, 15 present object slots and 15 Full/0 Sparse data points. The profile resolver accepts only the Flat7x seven-channel selector. Its Full-mode exception is limited to 15 present objects with one Full point each; sparse, absent-object and multi-point shapes remain rejected. The available fitter still supplies five rows and fails closed at the typed row-layout gate. No synthetic zero rows were added.

## 2026-11-15 isolated profile-separation increment

Feature worktree `C:\Users\chyinan\.codex\worktrees\JOCForge-isolated\flat7x-profile-separation`, branch `codex/flat7x-profile-separation`, starts at `2cdb18c50580b77b3bb8f60529ad96f7cf5ccbd5`. It adds separate profile fields for carrier codec route, substream topology, JOC reconstruction layout and ID14 payload `downmix_index`. Flat7x selects I0 legacy AC-3 BSID6 plus dependent D0 E-AC-3 BSID16, header index1, payload index1 and pair topology. Selected 5.1 remains independent E-AC-3 with payload index3 and its five-row fit.

FitRepresentationInput.row_layout distinguishes FiveChannel and SevenChannel coefficient inputs. The current fitter remains FiveChannel, so the Flat7x profile returns CoefficientRowLayoutMismatch before payload writing. The resolver admits only the matching Flat7x carrier/topology/selector tuple; 5.1 remains independent E-AC-3 with selector 3 and five rows. The Full-mode exception matches the observed 15-present, one-Full-point-per-object shape. No seven-row values, synthetic zero rows or new audio algorithm were added. Current verification: observed-shape gate 1/1, JOC reconstruction/profile integration 8/8, application Flat7x tests 6/6, selected 5.1 all-rate route 1/1, CLI check, and fmt check pass. No new encoder attempt was run on this worktree.

## 2026-11-15 Flat7x seven-row serializer seam

The second independent increment adds Flat7xCoefficientRows over caller-owned coefficient payload input. Admission requires selector 1, 15 present objects, one Full point per object, exactly seven channel rows per point, and each row length matching that object's band enum. The type's writer additionally requires the resolved Flat7x profile. The public profile-aware writer rejects selector/profile mismatches; profile=None keeps the existing 5.1 behavior.

The isolated parser maps mode 0/1 to FullCoarse/FullFine and 2/3 to SparseCoarse/SparseFine. The synthetic test fixture serializes 15 FullFine objects with one band across seven rows; the local parser reads 15 descriptors and 105 coefficient symbols. It is explicitly SYNTHETIC/TEST_ONLY and never enters production fitting. The existing fitter remains FiveChannel, so there is still no 7-row coefficient value producer or encode candidate.

Verification: coefficient_payload unit tests 2/2; protected_container unit tests 12/12; JOC reconstruction/profile integration 8/8; selected 5.1 all-rate route 1/1; CLI cargo check and fmt check pass. No encoding, OpenJOC, or full workspace test was run.

An additional selected 5.1 coefficient-fitting regression was attempted with cargo test -p jocforge-domain --lib coefficient_fitting::tests::three_generations_match_full_sparse_selected_and_all_score_call_counts --locked -j1. It stopped before coefficient serialization at RepresentationRetryProvider::from_approved_bytes with ProviderIdentity (coefficient_fitting.rs:558); the provider source and this path are outside this increment. The selected 5.1 all-rate route test passed separately.

## 2026-11-15 Flat7x application source seam

The selected 5.1 application path still uses CoefficientFitProposal and its 15x5 fitter. Flat7x now has a separate source entry accepting Flat7xCoefficientRowsProposal with validated 15x7 row shape, FitSourceLineage, and fit after-state. Source identity/generation is checked against the current projection; ID14 serialization then uses the existing dispatch provenance boundary. The profile-aware source entry without a rows proposal returns Flat7xCoefficientSourceUnavailable before running the five-row fitter. No 15x7 production provider is connected, and no encoding/OpenJOC run was started.

## 2026-11-15 Flat7x row proposal application seam

Flat7xCoefficientRowsProviderOutput carries validated row values. The application entry binds this output to the accepted QMF projection with Flat7xCoefficientRowsProposal::from_provider_output, deriving FitSourceLineage and a source-clock after-state that preserves temporal/geometry values while advancing generation to dispatch. The default profile-aware entry without provider output returns Flat7xCoefficientSourceUnavailable before running the five-row QMF fitter. The 5.1 profile keeps the current CoefficientFitProposal / FiveChannel route.

Verification: source row routing 1/1, row proposal same-source generation 1/1, Flat7x coefficient payload shape 2/2, application Flat7x tests 6/6, selected 5.1 all-rate route 1/1, CLI cargo check, fmt check, and diff check pass. The 15x7 producer remains unavailable; no encoder/OpenJOC run or candidate was created.

Verification for this incremental source seam: source-route tests 1/1; Flat7x rows proposal same-source after-generation test 1/1; shared dispatch provenance test 1/1; application route test 1/1; application Flat7x tests 6/6; selected 5.1 all-rate route 1/1; cargo check, fmt check, and diff check pass. Flat7x production remains unavailable without the authoritative rows provider; no encoding or OpenJOC run was made.

## 2026-11-15 Flat7x application bridge test

The public application seam accepts Flat7xCoefficientRowsProviderOutput, derives the proposal lineage from the accepted QMF projection, and crosses the common JOC dispatch provenance path. The application integration test uses a test-support identity-only projection plus SYNTHETIC/TEST_ONLY coefficient-row values; it verifies source generation, dispatch, producer transaction, and object count without a media/audio fixture. The authoritative coefficient provider remains absent, so the default Flat7x route still fails closed.

## 2026-11-15 Full-matrix parser order

Wash reports package openjoc_full_matrix_serialization_order_v1 (manifest 4ba2c0b6133267541a046e416b42adaf9b6c02aa79c98b987563f7ac3de70f7b; seal 20d2a61bc3aff599756526e19f7e1168310d31829a64c7ff36bdbb33323c3821): parser traversal is object → point/time slot → channel row → band; selector1 uses seven rows; band_index 0..7 maps to 1,3,5,7,9,12,15,23. The local Full serializer loop follows this order, which is a structural inference. No coefficient values or production rule are provided.

## 2026-11-15 synthetic ID14-to-AU structure test

The application bridge test uses SYNTHETIC/TEST_ONLY values for 15 present FullFine objects with one seven-row point each, serializes ID14 through the existing Flat7x source proposal, and checks selector 1 / object count 15. A separate frame test uses independent opaque SYNTHETIC/TEST_ONLY bytes for block-2 skipfld framing, D0 assembly and I0-to-D0 AU composition. It checks D0 chanmap 0x1A00, CI16 bytes 01 10, C<=511, atomic C=512 rejection, separate I0 640-kbps / D0 1024-kbps extents, aggregate 1664 kbps, and I0-before-D0 order in one AU. The skipfld fixture is not the ID14 body, a protected F13 container, or an EMDF parser-acceptance result. The tests remain nonpublishing; no candidate or real audio was created, and the authoritative production 15x7 coefficient provider remains missing.

Focused application bridge and opaque skipfld tests passed 3/3, the existing 15-object/seven-row parser round-trip passed 1/1, selected 5.1 all-rate regression passed 1/1, CLI check and fmt check passed. One stale test-only constructor call was aligned with the current provider/proposal API to enable that parser regression; production code is unchanged. Boundary check still fails on the existing F06 standalone_root.rs byte/hash inventory mismatch (expected 48,077 bytes; current branch file 49,292 bytes); this test slice does not touch that file or its map row.

## 2026-11-15 complete EMDF test-builder boundary

The existing test-support receipt gate does not build a parser-shaped F13 container from the current ID14 body. flat7x_receipt_gate_metadata_for_test creates a fixed 16-byte receipt-gate shell, and synthetic_receipt_gate_container_for_test explicitly bypasses parse_current_emdf_container with an empty payload list. The production EMDF serializer is private to MetadataOwner and is invoked from process_source_inner over queued ID1/ID2/ID11/ID14 bodies; no existing test-support API accepts a dynamic Id14BodyProposal and produces a full zero-slot container plus terminal layout. protect_emdf_container only protects caller-supplied zero-slot bytes. Therefore the ID14 serialization and opaque skipfld tests remain separate. A protected, parser-shaped EMDF containing this ID14 body and its parser acceptance remain NOT_RUN.

## 2026-11-15 OpenJOC AU0 carrier source-probe receipt

The earlier presence-only v0.20.1 carrier source probe reports D0 block2 and I0 blocks 3/4/5. Combined with the separate AU0 ID11/14 receipt it supported only a strong structural inference at that time. The later carrier-payload-id receipt below supersedes the inference for source-model ID-to-block association. Neither receipt observes original binary EMDF bytes; binary join remains UNBOUND.


## 2026-11-15 OpenJOC AU0 D0 block2 ID11/14 source-probe receipt

Wash single-file carrier-payload-id-report.json SHA-256 795684e1866159d0ff78480419b943bdf493aac10c278020f244b1003183994a is OpenJOC v0.20.1 tag-source MODEL evidence. One carrier record reports frame_owner D0/Dependent/substream0, block_index 2, skip_field_present true, classification PARSED, and payload_ids [11,14]. This is a direct association within the source-probe model and supersedes the earlier uniqueness-based inference for that model. The report still has binary_build_join UNBOUND, payload bytes/bit strings NOT_CAPTURED, and field_owner/generation UNBOUND. Public inspect alone does not expose payload-ID-to-block; this is not a direct observation of original binary EMDF bytes. The target still requires the complete EMDF container in D0 block2. Continuity report carrier-continuity-au1-4-report.json (SHA-256 1d235a51850be975315bc9de6e3ae9e2cade91bb8b2aa09e1770911d0ec268b8) reports the same D0/Dependent/0 block_index 2 parsed IDs 11/14 carrier in every AU1-4, with the AU4 boundary confirmed; together with AU0 this repeats across five AUs. This remains source-probe MODEL evidence only: binary_build_join is UNBOUND, payload bytes/bit strings are NOT_CAPTURED, PCM is NOT_RUN, and no original-binary or DEE output was observed.

## 2026-11-15 synthetic Flat7x paired-header contract test

The application structure test now uses existing complete I0 and D0 frame assemblers on synthetic opaque spans, then composes their actual builder output into the 6656-byte I0-to-D0 AU. I0 AC-3 header fields read back as fscod=0, frmsizecod=36 and BSID6, with six assembled block spans. D0 E-AC-3 prefix fields read back as strmtyp=1, substreamid=0, frmsiz=2047, fscod=0, numblkscod=3 and BSID16. Target remains I0 640/2560 plus D0 1024/4096, 48 kHz, six blocks, 1536 samples/frame. This is a SYNTHETIC/TEST_ONLY local writer-contract check, not binary/OpenJOC parity; its opaque skipfield bytes are not an EMDF container. The OpenJOC binary build join remains UNBOUND.

## 2026-11-15 F13/F14K block-2 protected-container source seam

CLI now carries the F14K ProtectedEmdfContainer bytes to the D0 frame builder. Before returning its D0 syntax summary, it verifies C is 1..511 and equals the protected container length, skip insertion follows the block-two prefix, the trailing block-two span is exactly the mantissa span, and the EMDF payload cursor shared with F12 is skipfield start + 10 bits. The F12 pair test succeeds only after matching those same protected bytes at the cursor. Test-support provider/wrapper/keyless JSON fixtures are normalized from Windows CRLF to canonical LF for their pinned identities; this is test-only. The focused source/container/cursor integration test passed 1/1, 5.1 all-rate regression passed 1/1, CLI cargo check and fmt passed. Its receipt-gate container is synthetic and bypasses parser acceptance; no full-length candidate ran, and no new Fresh GO was issued.

## 2026-11-15 D0 block-two preassembly source receipt

The single E-1037 Release run on `D:\Love Song 方大同.wav` completed Meter prepass (8,420 callbacks; selected LUFS -16.13749), then stopped after g=0/g=1 priming at `Flat7xCoefficientSourceUnavailable`. It exited 2 after 136,733 ms; the unique attempt directory is empty, with no candidate or partial file, so OpenJOC inspection did not run.

The block-two input now carries the Codec-owner-issued D0 generation receipt, the exact F13 lineage snapshot copied through F14K, F14K source/dispatch/AU identity, and the protected container. The constructor accepts an admitted F13 proposal plus the F13 lineage retained by F14K; it compares that complete typed lineage before assembly. Consumer and ID14 producer transactions, F13 owner generations, ID14 producer/writer generations, AnalysisOwnerGeneration, CodecOwnerGeneration, Flat7xD0SourceGeneration, and the later F15 Flat7xSourceGeneration remain distinct. Tests mutate owner generations, ID14 producer/writer generations, and analysis generation one at a time and confirm rejection before frame assembly.

OpenJOC selector1 defines target rows `[L,R,C,Ls,Rs,Lrs,Rrs]`, but the mapping from current 15x5x12 fit rows to seven coefficient rows remains UNBOUND. The zero-filled adapter was removed, and no synthetic rows are connected to production. The CLI continues to fail closed at `Flat7xCoefficientSourceUnavailable`. The selected 5.1 path is unchanged and not equivalent to DEE.

Focused verification: D0 receipt/frame assembler 8/8, D0 audio-frame 4/4, provider receipt proposal 3/3, application provider bridge 1/1, F13/F14K/F15 pair test 1/1, selected 5.1 all-rate route 1/1, workspace fmt/check. Workspace Clippy `-D warnings` reports 28 findings in unchanged files. Boundary check reports the unchanged direct-slice inventory for `crates/jocforge-domain/src/meter/standalone_root.rs`.

## 2026-11-15 protected F13 container and D0 block-2 cursor seam

The CLI published-source path passes F14K protected-container bytes to the existing D0 PCM frame builder. A fail-closed check now binds C to the protected container length (1..511), matches block-two skip insertion to the prefix receipt, requires the post-field span to equal block-two mantissa bits, and takes the payload cursor from Flat7xPairD0PcmFrameCandidate.emdf_container_start; that cursor must equal skipfield start + 10. F12 consumes the same cursor and verifies the protected bytes in the completed frame. The D0 audio-block builder already separates block-two prefix from packed mantissas; the skipfield lies between those spans. A focused pair/F12 integration test with SYNTHETIC/TEST_ONLY receipt-gate bytes passed after the assertions were added. Git-CRLF normalization in metadata test-support was needed for the hash-pinned JSON fixture on Windows; it changes only test inputs. No encoding candidate was started; Fresh GO is still required.

## OpenJOC structure audit for the next candidate

For any future candidate, use Wash's `D:\jocforge_wash_tmp\openjoc_flat7x_structure_audit_probe\run-audit.ps1` in a fresh attempt-local audit directory and the user-designated OpenJOC v0.20.1 binary. The default CLI component-rate JSON is stream aggregate and drops per-AU block index/malformed counts; it cannot prove the D0 block-2 IDs. The public walker API preserves per-AU `block_index` and payload IDs. Separate the gates: ID14 syntax, exactly one ID11 and ID14 with matching present group_id/profile, D0 as the last dependent carrier, and valid two-byte D0 Type-A addbsi. Complete absence requires examined=6, unresolved=0, malformed=0; unresolved/malformed later blocks are unobservable. This helper was not run because E-1037 produced no candidate. The standing authorization E-1052 applies to later attempts after the production provider and preflight are ready, using a separate registered attempt directory each time.

## 2026-11-15 seven-carrier input shape

`SevenCarrierPcmInput` is available from `Flat7xSourcePcmProjection` as a borrowed view in `[L,R,C,Ls,Rs,Lrs,Rrs]` order. The role map stores JOC bindings independently from I0 `[L,C,R,Ls,Rs,LFE]` and D0 `[Ls,Rs,Lrs,Rrs]`; the selected graph fixture maps JOC roles to physical rows `[0,1,2,4,5,6,7]`, leaving LFE row 3 and top-side rows on the physical N-row view. The role-map assumption remains provisional. This increment adds no QMF windows, carrier histories, object-slot lineage, coefficient values, or CLI provider. `Flat7xCoefficientSourceUnavailable` remains the production result. The one-hot mapping test, fmt check, workspace check, and diff check pass; Clippy and boundary check retain their documented existing findings.

## 2026-11-15 seven-carrier QMF window producer

`stage_flat7x_seven_carrier_windows` consumes the seven PCM slices from `Flat7xSourcePcmProjection`; each row goes through the shared `analyze_qmf_row` kernel with its own `QmfAnalysisHistory`. `SevenCarrierWindowSet` concatenates each role's prior 24 QMF slots with the current 24 slots into 48-slot raw-QMF windows, and retains the role bindings plus source/analysis/callback identity. The source receipt remains attached by `Flat7xSevenCarrierWindowCandidate`. No five-carrier mode-0 delay/FIR state is reused. The QMF state proposal is not installed by a programme owner, and no 15x7 coefficient fit or CLI provider exists yet; Flat7x still fails closed. Lane isolation/history rollover 2/2, mode-0 fitting 2/2, 5.1 all-rate route 1/1, workspace check, fmt and diff checks pass. Clippy has 27 existing findings; boundary check retains the unchanged `standalone_root.rs` inventory mismatch.

## 2026-11-15 C7 equation and solver primitives

The existing C5 normal equations, solver, normalizer, score and search have carrier-count-generic entry points with C=5 wrappers. A C=7 call computes 15×12 equation cells with 7×7 Gram matrices and 7-word cross/solution arrays; support growth follows `[L,R,C,Ls,Rs,Lrs,Rrs]`. The C7 fit now joins same-source target-slot and SevenCarrierWindowSet inputs, advances the JOC fit after-state, and writes selector-1 ID14 through representation retry using the actual Codec D0 source-generation receipt. Seven QMF histories are held in the JOC owner canonical extension; C5 state bytes remain unchanged. `source_execution` selects this path only for Flat7x. The first real candidate attempt should use the user's short ADM BWF `D:\testADM.wav`; OpenJOC inspection and playback remain pending. Existing solver/score fixture tests still fail identity/manifest checks.

## 2026-11-15 first OpenJOC render and ID14 capacity correction

E-1092 `D:\testADM.wav` produced a 137-AU experimental candidate. OpenJOC 0.20.1 parsed/admitted all AUs under observed-vendor compatibility and rendered an 8-channel 48 kHz 7.1 WAV. The rendered path is experimental with unresolved semantic binding. Wash's custom target check still flags OAMD `DynamicOnly + LFE` against its expected `Mixed bed + 15 dynamic`. Wash's scalar review confirms additional payload IDs are legal when exactly one ID11 precedes exactly one ID14; the probe's exact-two-item rule is too strict for `[11,14,2,1]`. ETSI strict is not a pass.

E-1093 on full Love Song completed the 8,420-callback Meter pass and stopped in F14/F14K at the first published source with `EMDF_PAYLOAD_CAPACITY_EXCEEDED`. The unique attempt contains no AU append or candidate. The C7 representation retry incorrectly used 511 bytes, which is the complete D0 skipfld container maximum; the ID14 inner payload parser caps at 372. The writer now uses 372 for ID14 while the D0 container bound remains 511. Next run the full Love Song target once in a new attempt directory, then use OpenJOC 0.20.1 inspect and render.

E-1095 with that capacity fix completed Meter and appended 19 AUs / 126,464 bytes, then stopped at local `EMDF_JOC_CHANNEL_INDEX`. OpenJOC 0.20.1 parsed and admitted all 19 partial AUs under observed-vendor compatibility. The local parser had applied the five-channel `Position5`/0..4 rule to selector 1; the approved generic seven-channel grammar supplies `Position7` and indices 0..6. E-1097 showed the initial U3 check still used `>4` after Position7/modulo7 were added; the follow-up now uses the selected carrier count for the table, modulus and range. C7 sparse retry is restored. The exact offending scalar was not captured. Full Love Song still needs a fresh complete run, then OpenJOC inspect/render and listening. Wash reports the OAMD `DynamicOnly + LFE` variant as one `Speaker(RcLfe)` anchor plus 15 dynamic anchors; the ID14-row-to-OAMD-slot identity join remains open.

E-1096 used the temporary Full-only retry policy and completed 513 AUs before C7 search returned `ScaledI32Range`. The first unrepresentable support candidate previously aborted search before other target-role supports were evaluated. C7 now skips only scaled-I32-overflowing candidates, continues the support traversal, and fails if none are representable; it does not clamp coefficients. Selected 5.1 behavior remains unchanged. Next action is a fresh full Love Song attempt.

## First full-length 7.1 render

E-1098 completed the full Love Song encode and EOF: 8,417 AUs, 56,023,552 bytes, 269.344 s, 1,664 kbps aggregate. OpenJOC 0.20.1 parsed and decoder-admitted all 8,417 AUs under observed-vendor compatibility; malformed=0, frame and metadata timing continuous. The full 7.1 render is `D:\JOCForge-attempts\E1098-love-song-flat7x-20261115-3f86ce3\render\love-song-flat7x-openjoc-7.1.wav` (48 kHz, 8-channel IEEE float32, 12,928,544 frames, LFE index3). This is an experimental render with unresolved semantic binding; no listening comparison has been made. ETSI strict reports ID11 wrapper flags and `codecdatae=0` on both ID11/ID14, while observed-vendor compatibility passes. The selected 5.1 all-rate profile/frame-size regression passed 1/1.

Next: user listening; align ID11 wrapper flags to official scalar comparison while keeping observed `codecdatae=0`; retain a same-AU/source D0 block-two placement receipt; resolve ID14 row to OAMD slot identity. The candidate remains non-production and not final-exposed.

## E-1100 wrapper and placement receipt

E-1099 was externally stopped at 3,217 AU / 21,412,352 bytes; OpenJOC parsed/admitted all 3,217 partial AUs under observed-vendor compatibility, malformed=0. E-1100 completed 8,417 AUs / 56,023,552 bytes / 269.344 s and EOF. OpenJOC 0.20.1 parsed/admitted all AUs, skip fields 8,417/8,417, malformed=0, timing continuous; ETSI strict reports only `codecdatae=0` for ID11/14. The OpenJOC render is `D:\JOCForge-attempts\E1100-love-song-flat7x-20261115-26d0454\render\love-song-flat7x-openjoc-7.1.wav` (48 kHz, 8ch IEEE float32, 12,928,544 frames, LFE index3).

The Flat7x wrapper overlay sets ID11 aligned=true, duplicate false/false, priority=0 and proc=0 while retaining codecdatae=0. The selected 5.1 wrapper stays unchanged. OpenJOC object scalar reports 16 ordered slots: index0 non-dynamic, indices1..15 dynamic; its report does not label index0 as `Speaker(RcLfe)`.

AU0 carries `D0Block2PlacementReceipt`: block 2, absolute D0 frame bits 12201..15979, PublishedAuIndex0, and Codec-owner D0 source-generation receipt 3. The separate F15 source-generation label is 2 (source-index-derived); these are distinct typed domains. Placement is retained from D0 frame candidate through F15. The Codec after-root still lacks an explicit comparison against that receipt, so the later root join remains provisional. The selected 5.1 all-rate regression passed 1/1.

## 2026-11-15 E-1104 output and ID1/ID2 composition

E-1104 (`b256d3e6e`) completed the Love Song Release run: 8,417 AUs / 56,023,552 bytes; all 8,417 D0 block-two placement receipts passed, maximum `C=474`, overflow=0. Candidate SHA-256 is `3887541587a42b5c69880192bf0fe7b6b8c8b1380e55ddf48fefd1842365a783`. OpenJOC 0.20.1 observed-vendor compatibility parsed/admitted all AUs, with continuous timing and no malformed AUs, and rendered an 8-channel 48 kHz float WAV at `D:\JOCForge-attempts\E1104-love-song-flat7x-20261115-b256d3e\render\love-song-flat7x-openjoc-7.1.wav`. It has 12,928,544 frames versus 12,928,512 coded samples; Wash's OpenJOC source trace attributes the 32-frame tail to `FinalLinkedGain` EOF drain. The Codec after-root now checks the expected generation from D0 placement against the actual Codec after-root, closing the provisional E-1100 join note above. Product exposure remains disabled; user listening and spatial identity remain open.

Wash-sanitized AU0 metadata for the user-designated MKA reports ordered payloads `[11,14]` across 23,033 AUs; E-1104 reports `[11,14,2,1]` across 8,417. The AU0 metadata file is `D:\JOCForge-attempts\official-demo-openjoc-20261115-03\AU0-METADATA.json`, SHA-256 `aabd2d359d3a1ace38e1180e8e1b26e2501a51ab08eb71df628fed8ef2a04b51`; its command log SHA-256 is `76a7340928e6e145c1cef3ea855df7ec00c0120254aa273b10689ff950d29b4c`. These files are not registered as a manifest-backed supplement in the isolated catalog. The registered `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` package (manifest SHA-256 `c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a`) records one ID11 and one ID14 per AU. Do not compare sorted/unique `access_units[*].payload_ids` with ordered `emdf.payload_orders`. The sanitized AU0 metadata reports `codecdatae=false` on ID11/ID14, so E-1104's ETSI strict failure on those fields is a shared observed value, not a candidate/reference difference.

Source trace confirms the extra payloads have typed inputs and data:

- ID2 (`AdditionalBsi`) is built by `metadata_bodies::assemble_metadata_bodies` → `id2_wire::build_id2_body` from the selected line/RF DRC profile strings plus the same-writer ID11 header's LFE flag. The builder also writes the selected phase and format values (1/1). It is 3 bytes / 21 valid bits. `film_light` maps to code 2; `none` maps to 0. This EMDF item is distinct from D0 frame addbsi complexity.
- ID1 (`Programme`) is initialized from a completed same-programme measurement in `Id1MeasurementConfiguration::from_measurement`, then `MetadataOwner::process_source_inner` advances `TimerState` by 1,536 samples and writes the timer output each source. E-1104 lengths are 2–3 bytes. The fields carry measurement/timer data, not blank defaults.

The local route's F13 contract explicitly produces and orders four bodies `[11,14,2,1]` (`METADATA-OWNER-ROOT-V1`, manifest `94774c412ce8457f3220d84e4b3386c0b48c21837d4a4943570791bb66f034b3`; body rules in `ID1-ID2-BODY-VALIDATION-V1`, manifest `25c85fee542bb4b68c039fc42d3ab686e0d7abe544fc5e32b6eaedc2d2967f1b`). Wash's OpenJOC source audit reports IDs 1/2 remain in generic EMDF payload/order/configuration/length summaries; only ID11/OAMD and ID14/JOC enter specialized parsing and `JocFrameInput`. Other decoder effects and target-specific standard optionality remain unresolved. E-1104 remains the four-payload baseline; the new Flat7x diagnostic mode emits `[11,14]` while retaining all four queue operations and ID1/ID2 pop provenance. The mode remains `production_allowed=false`; no acoustic or global decoder-effect claim is made. Wash also traced the native `RC_LFE` literal to an output write; row0-to-OAMD-slot identity remains `UNBOUND`.

## 2026-11-15 E-1105 and F15 Codec receipt ledger

E-1105 completed 8,417 AUs / 56,023,552 bytes with Flat7x experimental EMDF sequence `[11,14]`; D0 block-two placement receipts were 8,417/8,417, max `C=461`, overflow=0. OpenJOC 0.20.1 observed-vendor compatibility admitted all AUs, malformed=0, and frame/metadata timing were continuous; ETSI strict reported `codecdatae=0` for ID11/14. Its 7.1 render is 8-channel, 48 kHz IEEE float32, 12,928,544 frames. The render is available at `D:\JOCForge-attempts\E1105-love-song-flat7x-joc-pair-20261115-c2cc1e9\render\love-song-flat7x-joc-pair-openjoc-7.1.wav`; user listening remains open.

F15 input now carries the complete Codec `Flat7xD0SourceGenerationReceipt` from the D0 block-two placement; the application checks exact equality with the I0 candidate receipt, and the AU ledger retains the typed receipt. Domain validation binds its transaction, callback lineage, source index and dispatch to F13. F15 `Flat7xSourceGeneration` stays a separate label. Focused receipt tests, `cargo check --workspace --locked`, fmt and the selected 5.1 all-rate route pass. Clippy `-D warnings` remains red on existing diagnostics; `production_allowed=false`. Spatial role mapping and audible validation remain open.

## 2026-11-15 profile-aware ID14 U3 regression

The discriminator test directly calls `write_id14_header_for_profile` with `Id14HeaderInput.downmix_index=3`. Flat7x's resolved `joc_dmx_config_idx=Some(1)` serializes U3=1; selected 5.1's `None` preserves U3=3. The focused test and selected 5.1 all-rate route test pass. No production writer or audio path changed.

## 2026-11-15 F13 serializer to sequence-aware F14K parser

`flat7x_f13_serialized_bytes_roundtrip_through_sequence_aware_f14k_parser` stages source 2 through `MetadataOwner`, so the selected Flat7x F13 serializer produces the actual zero-slot bytes. Synthetic input-only OAMD and ID14 payloads are passed through F14K protection and `MetadataSourceProposal::bind_protected_container`, which invokes the sequence-aware parser. The test verifies F13 `[11,14]` serialization order, parsed payload order and bytes, declared/consumed container length, F14K protection ranges/codes, and zero terminal padding. The focused roundtrip and selected 5.1 all-rate regression pass; no audio is generated and production behavior is unchanged.

## 2026-11-16 E-1106 selected 5.1 PCM baseline

E-1106 used the same `D:\Love Song 方大同.wav` input and the default selected 5.1 route at 1,024 kbps; no Flat7x selector was passed. The Release run completed naturally with 8,419 sources / 8,417 published AUs, one 4,096-byte E-AC-3 frame per AU, six blocks / 1,536 coded samples per AU, and 12,928,512 coded samples total. Candidate SHA-256 is `bc01dcfc3b21cfefc4f20e522762e466d98d03375a14988d0b440d437f1db0a5`.

OpenJOC 0.20.1 observed-vendor compatibility admitted 8,417/8,417 AUs with zero malformed AUs and continuous frame/metadata timing. The inspect reports an I0-only 5.1 AC-3-mode frame, `acmod=7`, LFE on, six coded channels, and JOC profile index3 / five reconstruction inputs / 15 objects / CI16. ETSI strict reports ID11 `codecdatae=0`, `payload_frame_aligned=0`, and absent duplicate/priority/proc fields versus expected zero; ID14 `codecdatae=0` also differs from strict expectations.

The OpenJOC 5.1 PCM baseline is `D:\JOCForge-attempts\E1106-love-song-5.1-1024-baseline-20261116\render\love-song-5.1-openjoc.wav`: 6 channels, 48 kHz, IEEE float32, 12,928,544 frames / 77,571,264 samples, order `FL, FR, FC, LFE, Ls, Rs`. This is the comparison render for the E-1105 7.1 file; listening/PCM comparison remains pending.

## 2026-11-15 Flat7x selected-dispatch coverage

`flat7x_fit_crosses_selected_dispatch_and_writes_seven_row_id14` now runs the existing seven-carrier QMF window and fit stages, then directly calls `stage_selected_joc_source_with_seven_carrier_fit`; that production function calls `stage_joc_payload_with_seven_carrier_fit`. The test checks the fit's projected 15 objects each have one coefficient point with seven 12-band rows, the selected row layout/downmix index are C=7/index 1, and the actual written ID14 header has reconstruction index 1 and object count 15. Inputs use synthetic zero PCM and test identities; the test verifies dispatch wiring and serialized shape, not audio quality or authoritative coefficient values.

The selected 5.1 all-rate route test and the Flat7x owner-sequence test pass 1/1 each. The application F14K/D0/F15 equality test passes 1/1 using a test-only receipt-gate shell; the full E-1105 OpenJOC inspection will verify the real serialized container. `rtk cargo fmt --all -- --check` and `rtk cargo check --workspace --locked` pass. Workspace Clippy reports 29 existing diagnostics; boundary check stops on the existing `crates/jocforge-domain/src/meter/standalone_root.rs` inventory mismatch. The full workspace test suite was not run.

## 2026-11-16 Flat7x I0 source routing correction

The E-1105 OpenJOC internal-base decode has zero RMS on `L`, `R`, `C`, and `LFE`; its side and rear lanes contain signal. The E-1106 selected 5.1 base decode contains nonzero core-channel signal. Flat7x now builds a same-source six-lane `SourcedInstalledAudioFrame` from the provisional ADM I0 rows, reorders `[L,C,R,Ls,Rs,LFE]` to the existing F07 lane order `[L,R,C,LFE,Ls,Rs]`, and runs the existing F07-F10 and I0 AC-3 writer path. The selected 5.1 path still uses F04 final-six.

Focused row-packing tests, the nonzero F08 fullband/LFE numeric-plan test, selected 5.1 all-rate regression, workspace check, and format check pass. Workspace Clippy still fails on existing diagnostics; the boundary checker reports the pre-existing `standalone_root.rs` direct-slice inventory mismatch. Full Love Song Release re-encode and OpenJOC internal-base RMS verification remain pending. Provisional ADM-row role mapping remains explicitly named and replaceable.

## 2026-11-16 I0 BAP allocation and frame-budget fit

E-1109 confirmed that row routing alone did not restore audio. The first input and coefficient stages carried signal, but the initial I0 policy set q=0 and `caller_all_active_fine_zero=true` for blocks 1–5; those blocks returned zero BAP rows. E-1115 logged `[2693,95,95,95,95,95]` audio bits for the six blocks under that policy.

The Flat7x I0 caller now uses the same-source selected 5.1 core-frame q trial list as candidates for the separate 640-kbps AC-3 I0 writer. It tries the source-specific values in descending order, bounded above by the selected core q, and rejects candidates that exceed 20,480 audio bits or the 2,560-byte I0 frame. All six blocks use the selected candidate q with fine-zero forcing disabled. The selected 5.1 writer and its search are unchanged. If no same-source trial fits, source staging fails closed.

E-1120 used an active 256-ms segment of Love Song beginning at 30 s. The search selected q=400; block bits `[5146,2548,2548,2548,2548,2548]` total 17,886. Candidate SHA-256 `0d8db5b8e90039e53ebbaf2d27112362a05adb4b2c3e50bca0c39c8f3e0aaf58` is the same as E-1119's q=400 probe. OpenJOC 0.20.1 admitted 8/8 AUs with no malformed AUs. Internal-base RMS in `[L,R,C,LFE,Ls,Rs,Lrs,Rrs]` order was `[0.047170944,0.05005221,0.207763587,0.000000065,0.020916467,0.02042932,0.000002712,0.000003142]`; core L/R/C now decode with signal. The source segment has very low LFE energy, so full-song LFE behavior remains to be measured.

The targeted policy test and selected 5.1 all-rate regression pass. E-1121 completed 8,417 AUs / 56,023,552 bytes, candidate SHA-256 `81071d12024295e31e2215c2d4edd512e5110fb7fdffe0acb491a42d6a72566f`; block-two placement receipts are 8,417/8,417, max C=461, overflow=0, and all frame CRC residuals are zero. OpenJOC 0.20.1 reports deployed-compatibility pass and 8,417 stream-parsed AUs, while decoder admission is 8,416/8,417 and AU 6721 is marked malformed with `EMDF auxiliary data is not byte-aligned: 1004 bits`. Neighboring AUs 6720 and 6722–6723 pass. The E1121 D0 frame bytes are identical to E1105/E1109 throughout all 8,417 AUs. OpenJOC reports I0 as legacy-independent AC-3; the generic trailing 14-bit read at AU 6721 is 1004 versus 0 in E1105/E1109. Wash confirms ETSI TS 102 366 V1.4.1 §4.4.4.1 permits non-byte-aligned AC-3 auxbits and notes OpenJOC applies its E-AC-3 EMDF auxiliary classifier to each access-unit frame without a LegacyIndependent gate. The exact mapping from that I0 raw bit span to the AC-3 auxbits field remains `UNBOUND`, so OpenJOC's parser disposition alone does not establish a candidate syntax violation. Full OpenJOC decode/render stops at the parser error; full-song listening remains `NOT_RUN`. The ADM row-role mapping remains provisional and the candidate remains `production_allowed=false`.

An isolated parser-probe build from the exact OpenJOC v0.20.1 source tag (commit `eb5cdd5562017b7ec4d54bb243dd3a626310ecb9`) skips frame-end EMDF classification for LegacyIndependent AC-3 frames. The probe binary SHA-256 is `d7e8afc8b28349e23afbeaace51b8c1985ad0f88a832929d160345b4e729b355`; its patch SHA-256 is `45e222043b9fc470b0251b9a21f9c4273f00aa909ac54dfea9799b6a8b65298b`. With that validation-only build, E1121 parsed/admitted 8,417/8,417 and rendered a full 8-channel 48-kHz float WAV at `D:\JOCForge-attempts\E1121-love-song-flat7x-i0-budget-fit-20261116-8dddc9d\render\love-song-flat7x-i0-budget-fit-parser-probe-7.1.wav` (SHA-256 `be97c46a2fc00df5b61116b8155f00e33a931e7839aff6cdbf9405098a264f2e`). The internal-base core RMS is `[L=.03049982,R=.03160754,C=.08086876,LFE=.00557430]`; E-1106 comparison windows show a repeatable approximately -5,120-sample lag for L/R/C/LFE. This indicates the core signal is present and exposes a remaining source-time alignment question. The temporary probe is not the official binary, so its render is diagnostic, not official decoder acceptance.

Source-clock audit: `encode_request` binds reader rows at callback `g` to `first_sample=g×1536`; the selected programme clock maps that callback to `s=g-2`, and the publication clock maps source `s` to AU `j=s-2`. The Flat7x I0 path currently passes those current-callback physical rows into F07. Against corresponding source rows, E1121 internal-base windows peak at lag -4,864 samples for I0 L/R/C and -5,888 for D0 Ls/Rs/Lrs/Rrs; E-1106 core peaks at +256. This gives an approximately 1,024-sample I0-to-D0 offset, but the typed source-window-to-AU sample conversion remains `UNBOUND`; do not adjust it by the measured lag alone. Wash has been asked to bind the mapping from existing clock and source-extent contracts.

## 2026-11-16 I0 source-extent receipt

The Flat7x-only I0 analysis input now carries the validated reader extent and requires exact equality with the same callback's D0/JOC projection. That extent remains attached through the I0 frame and paired AU candidates, and the F12 I0/D0 join checks equal extents. The shared selected 5.1 F04 route is unchanged. The receipt binds the consumed reader window; `first_sample` to AU `j` remains `UNBOUND`, and no row shift was made.

Verification: workspace check, format check, focused extent equality test, and selected 5.1 all-rate regression passed. The feature-gated pair integration test stopped before the new extent assertion because its synthetic q=400 I0 allocation needs 22,518 bits against a 20,480-bit budget. No PCM/q/audio policy change or encoder run was made. Continue with the approved source-clock mapping review; keep timing conversion separate from the encoded AU index.


## 2026-11-16 E-1123 full-song candidate

A one-shot Wash Fresh GO produced the candidate with Release commit add93721527c3aa915e8729de0ecb5d69cac6476 from the designated Love Song input. The run committed 8,417 AUs / 56,023,552 bytes, with D0 block-two placement receipts 8,417/8,417, max C=461, overflow=0 and AU CRC residuals 0. Candidate SHA-256 81071d12024295e31e2215c2d4edd512e5110fb7fdffe0acb491a42d6a72566f exactly matches E-1121, so the metadata-only source-extent change did not alter encoded bytes. The attempt log also records 7 pending drop receipts, programme_done=false, production_allowed=false; render and playback remain NOT_RUN. E-1123 was not rechecked with OpenJOC; the existing OpenJOC 0.20.1 result is for the same exact candidate hash: decoder admission 8,416/8,417, AU6721 flagged for 1004-bit auxiliary-data alignment. No new listening render or subjective result exists.


User listening feedback: E-1123 sounded as bad as the previous version. Its candidate hash matches E-1121 exactly, so the new metadata-only extent commit left encoded audio bytes unchanged. No numerical similarity score or new render was recorded.

## 2026-11-17 DEE timebase sample-range boundary

Wash-only static receipt dee_atmos_ddp_timebase_sample_range_consumer_wash_v1 (manifest 553f7c4764628a885d062b820038254887fdc529cc9c7503e7c7de57eeb58336; seal 1166524d...; normal/O verified) records that the DEE file_position branch subtracts offset seconds from start/end seconds, uses round(sample_rate x seconds) for validation, and truncates the adjusted products to signed-64 sample positions. The embedded_timecode branch preserves start/end seconds without offset subtraction; FFOA remains a separate seconds field. The first source-range consumer is FUN_180145380; its second caller was not followed. The mapping from those sample bounds to AtmosReader cursor limits, JOCForge first_sample/source/AU coordinates, and endpoint inclusivity remains UNBOUND. E1123 bad listening result therefore still has no justified sample shift rule.

## 2026-11-17 DEE loader-list boundary

The attempt-209 Wash package (manifest 55f497137061217b0c0bbcf238b217c858c778c13e3d607acff09e47526acbe2; seal 73c4b8a90de9c677779115dccbab37263cae28c016507ba58123572514e20b70; normal/O pass) binds the 7.1 test host default root to D:/Programs/dee71joc/test_dee and the default list path to D:/Programs/dee71joc/test_dee/dee_loader.list; the file is absent. Host global context +0xF8 can override the caller default and the chosen string is written to path collection +0x40. The separate semicolon-list parser input has no typed join to that collection/file. The filter wrapper census has no code consumer, so filter registration and selected-child identity remain UNBOUND. Attempt-208 GO remains consumed; no run or config change was made.

## 2026-11-17 Geometry descriptor and R8 lineage

Wash-only Ghidra receipts on candidate 2b7 bind the selected writer path as follows: FUN_1402ac760@0x1402ac78a passes a stack address in R8; its LEA R8 base is RSP and the exact slot normalizes to function-entry RSP-0x58, with no direct store or initializer before the call. The alternative FUN_1402ad5b0 path has R8 live-in, one .pdata DATA reference, and no validated direct E8 caller. In FUN_1402ad5d0, R8 is preserved through R15 and restored before CALL@0x1402ad674 to FUN_1402ad310. That callee reads an 8-byte qword through [R8] and writes it to the RDX stack descriptor at offset 0. The native owner/generation, R8 pointee contents, descriptor field type, and D0 row join remain UNBOUND; no record bytes were read.

## 2026-11-17 D0 record-vector and skipfld boundaries

Wash address lineage binds record-vector begin at worker_state+0x190 and end at worker_state+0x198; a CMP compares them, and the unselected fallthrough writes byte 1 at begin+0x10, increments the begin pointer by 0x40, then compares it to end. The comparison outcome and record-field semantics remain UNBOUND. Separately, the selected skipfld writer path reads U16 count and U64 source from an opaque record pointer at FUN_14074a990.param_1+0x10cc; U9 assignment, D0 block ordinal, complete C, capacity, and native overflow disposition remain UNBOUND. The target contract stays one D0 block-2 skipfld, C<=511, reject overflow, no cross-field continuation.

## 2026-11-17 audio identity and D0 record boundaries

The user reported that E-1123 sounds as bad as the previous version; its candidate hash is identical to E-1121, so encoded audio bytes did not change. A separate keyless candidate file matches its generation receipt by hash and size, but its Python run/source binding is UNBOUND. The Rust failure artifact is USER_REPORTED_FAILURE_FILE_ID_UNBOUND.

DEE timebase static evidence now binds file_position start/end seconds minus offset, rounded validation and truncating signed-64 sample positions; embedded_timecode and FFOA are separate. The join from those sample bounds to the JOCForge AU clock remains UNBOUND.

The D0 record-vector begin/end path compares worker_state+0x190 and +0x198; its unselected fallthrough writes byte 1 at begin+0x10, increments the pointer by 0x40, then compares to end. Record/row semantics remain UNBOUND. The skipfld writer path reaches an opaque record pointer at FUN_14074a990.param_1+0x10cc; native U9/skipfld assignment, D0 ordinal, C and capacity remain UNBOUND.
