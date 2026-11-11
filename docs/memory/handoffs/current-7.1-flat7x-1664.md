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
- `JOCFORGE-EAC3-CHANMAP-U16-SERIALIZATION-V1` 的 numeric helper 保留在 `codec::eac3_channel_map`。后续 Wash supplement `eac3-dependent-chanmap-field-writer-v1` 增加同模块的 pair helper：先写 `chanmape=1`，再写 typed 16-bit MSB-first `chanmap`，完整预检 17 bit。16 个 one-hot、target `0x1A00`、start bit 5 misalignment 与 16-bit capacity reject vectors 本地重放通过。helper 不接入完整 D0 BSI/frame/AU path。
- The pair supplement's `acmod=6,lfeon=0` is an independent validation-vector context; the Flat7x D0 target tuple is `acmod=5,lfeon=0` as confirmed by the fixed-prefix contract. The field-pair helper consumes neither context.
- Wash package `eac3-dependent-d0-fixed-prefix-v1` adds `codec::frame_builder::write_flat7x_d0_fixed_prefix`. It preflights and emits exactly the target's 45-bit prefix, ending at bit 45; the caller owns `dialnorm` and remaining BSI. The chanmape/chanmap pair stays a later, separate BSI field group. Its corrected capacity vector describes a 5-byte / 40-bit buffer and verifies no partial write. The helper does not compose a full D0 frame or enter production.
- Wash package `eac3-dependent-d0-bsi-segment-v1` adds `codec::frame_builder::write_flat7x_d0_bsi_through_chanmap`. It composes prefix + caller-supplied dialnorm + compre/present optional compr + chanmape/chanmap and returns before caller-owned `mixmdate`. The full segment preflight is 68 bits without compr and 76 with compr. dialnorm 0 and 32 reject before mutation. No XML DRC translation, compression algorithm, complete frame or production path is introduced.
- Wash package `eac3-dependent-d0-mixing-metadata-v1` adds `append_flat7x_d0_mixing_metadata_and_infomdate` after the parent cursor 68/76. It serializes explicit optional mixing levels, six optional block values, and infomdate; it returns immediately after infomdate and preserves later information metadata for the caller. No XML mapping or audio algorithm is added. The package capacity vector says 16 bits remain at cursor68; the byte-backed `FrameBitWriter` exposes 12 or20 there, so local atomicity testing uses 12 and awaits Wash's vector correction.
- Wash package `flat7x-i0-ac3-header-segment-v1` adds `codec::frame_builder::write_flat7x_i0_ac3_header_through_compression` for the independent I0 legacy AC-3 path. It emits only the header through optional `compr`, accepts frmsizecod 36/37, and returns at the caller-owned langcode boundary after 62/70 bits. CRC1 and the DRC fields are caller-supplied values; this helper computes no CRC and maps no XML DRC profile.
- I0 reference value is provided by `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1`: the user-designated sample uses legacy independent AC-3 BSID6 at 2,560 bytes. The target-specific header segment through `compr` is now locally materialized; the complete legacy-core I0 frame writer/callsite and audio blocks remain open. The selected 5.1 prefix remains E-AC-3 BSID16/`strmtyp=0`, unchanged.
- 已按新的 partial Wash package，在现有 1024 kbps / 5.1 core profile 上实现 immutable `JocReconstructionLayout::Flat7x` 与 3-bit `joc_dmx_config_idx=1` header field。mode0/phase1/internal index3 和 5.1 coefficient input path保持独立；profile-aware writer 的 field-order 和 baseline byte tests 通过。
- 已核对的 CLI writer 链为 `run_with_data_rate_and_joc_layout` → `run_worker`/`run_inner` profile resolution → callback `stage_source_frame` → `prepare_main_metadata_bodies_for_callback` → `stage_selected_joc_source_with_transaction_for_profile` → `stage_joc_source_with_transaction_for_profile` → `stage_joc_payload_for_profile` → `write_id14_header_for_profile`。该调用链的对象是现有 selected 1024 kbps full-file profile；`--joc-layout flat7x` 将 index 1 送到 ID14 writer，省略选项的 5.1 writer bytes 与既有 writer 完全一致。
- 上述调用链对应 selected 1024 kbps profile；`Flat7xI0D0TargetPlan` 保持独立的 1,664-kbps target model。target D0 侧已有独立的 `d0_emdf_skip_field` helper；下一连接点是由 D0 frame composition 调用 profile-aware ID14 producer，并传入 1024-kbps D0 context。selected 5.1 carrier 路径与 D0 composition 分开。
- 较早的 partial native transaction package 没有证明 I0/D0 writer identity、target order、same-generation lineage 或 atomic publication。

## 开放边界

- D0 source-row-to-semantic-role mapping：`UNBOUND`。
- Flat7x I0 writer mapping to the observed legacy AC-3 BSID6 reference：`NOT_IMPLEMENTED`；the existing selected 5.1 E-AC-3 BSID16 path remains separate.
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

The caller still owns frame cursor placement and block/frame composition. Native block-2 cursor, D0 BSI/audio owner, source/generation lineage, all-AU fit and product output remain `UNBOUND/NOT_RUN`; `production_allowed=false`.

## 2026-11-11 official Atmos reference structure

Wash package `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` passed both checker modes and is registered at manifest `f7f8630c3f18e990324f013e70486c561de6816a844f852f18f03a82ce28fbfc` / seal `2d883e278bb83fa0d617610f94b8a87fa5849ac7bf19bdf08818d8d30d50740e`. Its 23,033 packet summary reports I0 legacy independent AC-3 BSID6 / 2,560 B followed by D0 dependent E-AC-3 BSID16 / 2,560 B, 1,280 kbps aggregate. D0 uses `chanmap=0x1A00`, a two-byte addbsi/CI16 record, and block-2 EMDF IDs 11/14 with observed skip-field lengths 301–377 B. The user target remains I0 640 + D0 1,024 = 1,664 kbps, with a 4,096-byte D0 frame; the sample's 640-kbps D0 and target are distinct. No media/audio bytes are included, and the package keeps `production_allowed=false`.

Wash addendum `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` (manifest `c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a`, seal `0fb4c7f8a7a49488a97586b7e3eb6febc0542cf0044569fe37e65dc07d85757d`) reports `joc_dmx_config_idx=1`, extension index 0 and 15 JOC header objects on all 23,033 AUs. This binds the reference sample's profile index; it does not establish a native target writer or profile-index 1 on a new 1,664-kbps output. Payload IDs 11/14 have `codecdatae=0` while ETSI Table 56 requires 1; vendor-compatible observations and ETSI-strict failure remain separate, and no strict-conformance claim is made.

The new reference identifies the intended Flat7x I0 structural value as legacy AC-3 BSID6. The profile-specific I0 writer path still needs its own implementation; preserve the existing 5.1 E-AC-3 BSID16 path. The D0 skip-field serializer remains a field helper, without a native block cursor or frame builder callsite.

## 2026-11-11 official Atmos OAMD position reference

Wash package `OFFICIAL-ATMOS-DEMO-OAMD-POSITION-WASHED-V1` is registered at manifest `c550a0536ed4752b46872c674fcabc8ad8828b7eb69e17e6fb4a0d082ee52ef8` / seal `906f48f17f47ef372d339bebe598931a933e6dc132452862ab630fce4582c5f5`. Its normal and `python -O` checkers pass. The user-designated 1,280-kbps sample has 23,033 AUs, one bed/ISF slot and 15 dynamic-object slots. Each dynamic object has one absolute room-position/Z update per AU; nine objects use Z code 0 and six use positive ceiling code 15. The `b_enable_elevation` zone constraint is true on all 345,495 updates. Separately, `object_size_idx=0` on all 345,495 updates and resolved object-size height is zero across 345,510 states.

The sanitized target reference fixture is `crates/jocforge-domain/tests/fixtures/flat7x_metadata/official_demo_oamd_position_reference.json`. It keeps position Z, the elevation-zone constraint, and object-size index/resolved height in separate fields. It contains aggregate counts only; coordinates, source-row mapping, target metadata recipe, and encoding behavior remain open. The sample is 1,280 kbps while the 1,664-kbps target is `NOT_EVALUATED`; `production_allowed=false`.

## Flat7x reuse and target-native gap matrix

| Component | Existing reusable structure | Flat7x target-specific requirement | Evidence and status | 5.1 regression gate |
|---|---|---|---|---|
| Main sealed reference | Ordered I0→D0 AU shape and `OwnershipPlan` architecture shape | No frame syntax, rate allocation, BSI value, or D0 writer is inherited from the reference | Architecture shape only; runtime/writer identity `UNBOUND` | No 5.1 source is reused or changed |
| Old map / split-rate / legacy D0 | None for the new target | Old `chanmap=0x0200`, old 1,024+640 split and legacy D0 writer stay out of the Flat7x row | Legacy/reference paths remain separate evidence | Existing 5.1 map and writer behavior stays fixed |
| Isolated 5.1 path | Six-rate resolver, selected F12 E-AC-3 frame writer, `OutputOwner`, and durable atomic sink remain the independent baseline | Flat7x additions stay additive; no global frame-prefix or rate-row rewrite | Current 5.1 path remains implemented and locally regression-tested | Run `selected_frame_trial::tests::supplied_absent_payload_plan_builds_only_a_complete_keyless_scratch_frame`, `output_owner_rate_profile`, and `prepared_adm_input_preserves_each_washed_rate_through_profile_and_frame_size` |
| I0 | Existing bitio primitives plus the additive Flat7x legacy AC-3 header segment writer | Legacy independent AC-3 core BSID6, 640 kbps, 2,560 bytes | Header segment through optional `compr` is locally verified at 62/70 bits; CRC1 is caller supplied. Complete I0 body/frame writer, audio blocks, runtime source binding and output remain open. | Keep selected 5.1 E-AC-3 BSID16 path unchanged |
| D0 | Existing bitio primitives, fixed-prefix/BSI/mixing/information segment writers, typed BSI composition, six-block layout proposal, caller-supplied frame-bit assembler, and separate skip-field serializer | Dependent E-AC-3 BSID16, 1,024 kbps, 4,096 bytes, `chanmap=0x1A00`, one block-2 complete EMDF skip field | Prefix/BSI/metadata structure and synthetic six-block layout have local tests. The frame-bit assembler copies only caller-supplied opaque spans, block-two EMDF bytes, and caller CRC2 into a 4096-byte scratch frame; vectors are `SYNTHETIC_NOT_REAL_OUTPUT`. It runs no audio/mantissa/BAP/CRC algorithm and does not establish a playable or real frame. Full frame acceptance, native block cursor, AU/source role and production remain open. | Existing 5.1 header path remains unchanged |
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

## 2026-11-11 caller-supplied D0 frame-bit assembler

`assemble_flat7x_d0_frame_bits` stages the approved 4096-byte D0 frame using caller-supplied opaque audio bit spans for six blocks, one block-two EMDF container bracketed by caller-supplied pre/post spans, and caller-supplied CRC2 U16. The explicit block-two insertion offset remains caller-owned/unbound. It requires the BSI, six spans, single skipfield, and CRC2 to total exactly 32768 bits; source extent, output capacity and placement checks precede the single caller-writer commit. Synthetic test patterns are explicitly `SYNTHETIC_NOT_REAL_OUTPUT`; they do not establish playable output or audio/DEE equivalence. No audio algorithm, mantissa/BAP generation, CRC calculation, block3 continuation, AU/GTC or production call is connected.

## 2026-11-11 Flat7x target ID14 header adapter

`write_id14_header_for_flat7x_target` requires the approved `Flat7xI0D0TargetPlan` and semantic object count 15, then invokes the existing profile-aware ID14 writer with the Flat7x reconstruction index 1. The writer's approved U6 count-bits field is `15 - 1 = 14`; mode0/phase1/internal index3, complexity16, seven reconstruction carriers, 15 reconstruction rows and ADM physical input rows remain distinct. The header-only input has no body field. Missing/invalid target contracts and object-count mismatch reject without changing writer state; the existing 5.1 header byte-equality regression remains in the same suite. This adapter emits no D0 audio or complete 1,664-kbps AU/frame and does not close production or parser/W04 acceptance.

## 2026-11-11 opaque I0→D0 AU byte composer

`write_flat7x_i0_d0_access_unit_bytes` accepts typed opaque I0/D0 frame byte spans, checks the approved 2560/4096/6656-byte extents, requires writer cursor zero and preflights all 6656 bytes. It stages I0 followed by D0 and commits once, then returns a noncommitting composition-layout receipt containing only byte spans/extents. BSID, CRC, source-generation identity and AU semantic metadata remain outside this slice. The vector uses synthetic fill bytes marked `SYNTHETIC_NOT_REAL_OUTPUT`; the selected 5.1 path is unchanged.
