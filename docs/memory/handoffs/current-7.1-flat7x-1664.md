---
status: current
topic: 7.1-flat7x-1664-handoff
spec: JOCForge
confidence: package-backed-target
last_updated: 2026-11-12
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

The Wash-sealed `FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1` (manifest `689d5e4f3b27fd79110f6fb99189d4407384424929b3a74d20ba3447698f545b`, seal `5f0c525745784e7b41714d04e5dc432fc2689ccf6ae74e925afe3cf7662e82b6`) now has a typed implementation seam. `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits` travels on `Flat7xD0AudioSpanReceipt` through pair staging to the AU/frame composer. The writer emits a zero `auxdatae` bit, zero-fills the remaining frame-level auxbits after the six audio blocks and block-2 EMDF skip field, and reports the span/padding before final CRC2. The assembler enforces one complete block-2 skip-field and existing 1–511 byte bounds. Arbitrary opaque tail input is gated by `test-support` and retained only for a synthetic structure-copy test.

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
