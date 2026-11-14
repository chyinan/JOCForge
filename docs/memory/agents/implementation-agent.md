---
status: partial-implementation-locally-verified
topic: implementation-agent-status
spec: JOCForge
confidence: repository-status
last_updated: 2026-11-14
tags:
  - implementation
  - handoff
  - 7.1-flat7x
---

# 实现 Agent 状态

## 当前状态

- 隔离分支为 `experimental/independent-encoder`。本 slice 之前的远端基线包含 Flat7x CLI selector/profile/header plumbing 与 F13 typed validation reason；本次 target-plan 变化单独交付。
- 新增 `codec::access_unit_composer::write_flat7x_i0_d0_access_unit_bytes`：固定 I0→D0 opaque byte composition，校验 2560/4096/6656-byte extents、zero cursor 和整段容量后一次写入；synthetic-only、`production_allowed=false`，未连接生产路径。
- 新增 `codec::elementary_stream_composer::compose_flat7x_elementary_stream`：按序组合 caller-supplied opaque 6656-byte AUs，checked size、byte-aligned cursor 与整段容量预检后 private staging 并一次写入；空序列拒绝，synthetic-only、`production_allowed=false`，未连接文件/GTC/product encode。结构测试已串接 D0 frame→AU→双 AU stream 并验证 receipt-derived D0 offsets 与 CRC2尾部保持。
- 新增 `Flat7xElementaryStreamStageProposal`→F15 `Flat7xAuFinalizeProposal` ledger join：逐 AU 核对index/offset/extents与frame/AU hashes；IO receipt candidate绑定同一attempt/owner generation并仅在flush/sync/readback一致后生成stage-after proposal。caller-owned temp file失败需废弃；final path/rename/GTC/product仍关闭。F15 finalize-ledger join用synthetic model通过；真实F13/Integrity正向owner产物、source callback及public File-adapter调用仍 `NOT_RUN`，`production_allowed=false`。
- Wash E-591已把 first-output preflight target绑定到 Love Song 方大同.wav（1,396,237,802 bytes，SHA-256 `34ecda6afa708aaf436930b86cc8d120a73faff2847088be975b10a22039f8b5`）及候选 `.ec3`/temp路径；两目标当时均不存在。Fresh GO仍未收到，当前不启动编码；因I0/D0真实帧与F13/F12→F15 finalize owner链未闭合，12:00前输出不可信。
- 项目指引记载 product encode entry 仍不可用；独立 validation 和 W04 acceptance 为 `NOT_RUN`。
- 1664 kbps I0/D0 AU profile 保持 `production_allowed=false`；1024 kbps / 5.1 Flat7x profile/header partial slice 已进入 resolver 和 writer API，完整文件验收为 `NOT_RUN`。
- `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` provides the user-designated I0 reference: legacy AC-3 BSID6 / 2,560 bytes. The new Flat7x I0 assembler composes caller-owned BSI/audio spans and generates CRC1/CRC2 with synthetic zero-residual receipts. Audio-block generation and a typed source callsite remain open; the selected 5.1 E-AC-3 BSID16 path stays unchanged.
- `codec::ac3_bsi_tail::write_flat7x_i0_ac3_bsi_tail` now serializes the Wash-bounded I0 BSI-tail optionals from header cursor 62/70, omits dual-mono fields for acmod7, validates U5/U2/U14/U6 and addbsi 1..64-byte bounds, and returns exact tail/payload extents. Four synthetic tests cover minimum, all optionals, max addbsi and atomic rejects; `production_allowed=false`.
- `assemble_flat7x_i0_ac3_frame_from_fields` adds a typed composition entry over the compatible opaque-span assembler. It serializes BSI privately, passes its receipt span with six caller-owned audio spans, and returns BSI/frame receipts only after whole-frame CRC checks. Two synthetic integration cases cover successful composition and fail-before-write rejection; no audio or production caller is added.
- I0 CRC contract `FLAT7X-I0-AC3-CRC-GENERATION-ETSI-V1` is registered at manifest `07ad2b60bd22a1312ff15107cf35e836a12a763c764b3fbaa74e02b4f2d9dc2d`, seal-file SHA-256 `14020e164f7805723a0fd90cd77f924961a3a457315f8fdbe4219dafdaccb031`. Local coverage includes CRC1 target/aligned/odd-boundary scalar values, CRC2 generation and zero residual, and the scalar crcrsv collision predicate. Wash collision bytes are absent, same-byte replay is `NOT_RUN`, and `production_allowed=false`.
- Wash 已授权并封存 `JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1`。本地已增加 immutable `JocReconstructionLayout::Flat7x` profile row 与 profile-aware 3-bit header field writer；mode0/phase1/internal index3 与 coefficient downmix index3 保持原值。输入型测试覆盖 index1、field order、profile separation 和 baseline bytes。
- `ATMOS71-FLAT7X-1664-IMPLEMENTER-HANDOFF-V1` 已物化为 value-only `Flat7xI0D0TargetPlan`、候选 AU metadata validator，以及检查 frame extent/CRC 并按 I0→D0 拼接的内存 byte-sequence candidate。9 项定向 domain 测试通过；现有 5.1 resolver 回归通过。该代码不解析 frame syntax、不产生 frame、不接受生产 profile，也不创建 writer 或 publication authority；1664 kbps target 的 `production_allowed=false`。
- 新增 target-only `Flat7xAuOutputOwner`：stage 要求 F13 `MetadataSourceProposal` 与有序 I0/D0 `IntegrityF12SourceReceipts`，并校验载荷、身份字段、frame extent/hash/CRC、D0 block2 EMDF 容量。owner generation、PublishedAuIndex、F13 source/AU 和 caller-supplied source-generation label 分别保留。missing receipts / incomplete finalize 产生 attempt retirement；finalize proposal 只在完整 AU count/ledger 后生成，仍无 GTC/final expose 权限。
- 独立 `jocforge-io::flat7x_au_append` adapter 现可对调用方的 staging `File` 检查偏移、写入 aggregate AU、`flush`/`sync_all` 并精确读回后返回 receipt candidate。失败可能留下 staging bytes，caller 应废弃 attempt；adapter 无 rollback、final exposure、GTC 或 encode 连接。
- Wash `FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1` 已在本地验证：source-generation label、PublishedAuIndex、F13 source/AU identity 和本地 owner generation 分开保存；缺少 required F13/I0/D0 integrity receipt、staging fatal 或 incomplete finalize 都 fail-stop/retire。F13/Integrity pair positive integration、callback owner join、GTC canonical output root 与 final expose receipt 仍未运行。
- Wash `FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1` 已按 manifest `1034ef415113b666acc5aefaa3207230040f05d9ebc05a33933bffc50b4ff088` 与 seal `16d925352e9b197e016b2554f9c2ae609e8f8890e3dca81c4eb591f737b41210` 校验并登记。F15 gate 的 synthetic typed receipt 正向配对与身份、carrier、容器尺寸拒绝用例通过；fixture bypasses F13 parser and F12 owner stages，`production_allowed=false` 与 source/callback generation join `UNBOUND` 保持。
- Flat7x D0 block-2 skip-field syntax helper requires a complete nonempty EMDF container and writes `skiple=1`, U9 byte count, then exact bytes through `FrameBitWriter`; missing input returns before any bit is written. The helper is connected to `D0FrameBitAssembler`, which passes the caller-owned block-two container at the layout-derived field span; native block cursor/source-owner binding remains unbound. Wash `FLAT7X-D0-SKIPFLD-STRUCTURE-V1` passed normal and `-O` checks (manifest `1b14343db4128bda45c254a024de2662ac59516e2e83aa7d3191e3c4d80ca1c3`, seal-file `7c31c485368c3755de23f7fee981a141c3607eef86d7110a4cdf1661c6a15024`); official sample C_total/skipl is 301..377. C511 is arithmetic FIT_EXACT only; full OpenJOC C511 parse remains `NOT_RUN`, and `production_allowed=false`.
- Wash `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1` is registered at manifest `f7f8630c3f18e990324f013e70486c561de6816a844f852f18f03a82ce28fbfc` / seal `2d883e278bb83fa0d617610f94b8a87fa5849ac7bf19bdf08818d8d30d50740e`. It reports reference I0 legacy AC-3 BSID6 then D0 dependent E-AC-3 BSID16, each 2,560 B, across 23,033 packets; this 1,280-kbps sample differs from the 1,664-kbps target's 4,096-B D0. The target-specific I0 BSID6 writer remains a separate open slice; the existing 5.1 multirate writer stays unchanged.
- V2 addendum `OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2` (manifest `c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a`, seal `0fb4c7f8a7a49488a97586b7e3eb6febc0542cf0044569fe37e65dc07d85757d`) reports sample `joc_dmx_config_idx=1`, extension index 0 and 15 JOC header objects on all AUs. IDs 11/14 have `codecdatae=0` while ETSI Table 56 expects 1; vendor-compatible observation and strict failure remain separate. Target writer/index binding, strict conformance and 1,664-kbps output remain open.
- CLI selector 增量已按 `JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1` 接入：可选 `--joc-layout flat7x` 经 immutable resolver/value path 传至 JOC header；省略参数保留旧 1024/5.1 默认。source-row 与 audio state 不新增 owner。
- E-521 获准的 Love Song N36 / 1024 kbps Flat7x 单次运行已结束：release binary SHA-256 `098380d51069293dcf9b83b3165e3bb7235d598c1d57f24184d147c87811690d`，attempt `1794325737775022700`，exit code 2。stderr 报告 `F14/F14K source stage: MetadataOwner(CurrentEmdfValidation)`；没有发布最终 EC-3 文件，JOC internal fields、FFprobe 和 official-demo structure match 保持 `NOT_RUN`。完整本地收据位于 `.test-tmp/love-song-n36-flat7x-1024-20261110/run-receipt.json`。
- 已在 F13 MetadataOwner 增加 non-payload `CurrentEmdfValidationReason` discriminator；稳定错误码和 reject 条件保持不变。reason 在 source-stage Debug 结果中可供后续 run receipt 记录，E-521 旧日志仍不能反向辨别具体 predicate。
- Format 与 workspace check 通过；boundary check 为 78 locally verified scoped tasks / 104 IDs / 0 unmaterialized roots / 6 partial roots。Flat7x、新 profile gate、ID14 header framing 和 coefficient payload targeted suites 共 15 项通过。
- Workspace Clippy `-D warnings` 仍报告 9 项既有 analysis、DRC、metadata-owner 与 programme-state lint；target-plan 模块修正 excessive-bools 后未出现在报告中。Cargo 同时报告全局 last-use 数据库磁盘已满；未清理或删除任何缓存。当前只运行该 slice 的 7 项定向测试，未重跑完整 workspace tests。

## 下一步

- E-521 prelaunch 与失败收据已回传 Wash；source predicate matrix 也已对照 F13/F14K/F15 contract 发回。精确 failed predicate 仍 `UNBOUND`，新增 reason 仅改善未来授权运行的定位。
- 当前单次 attempt 没有最终输出。FFprobe 外层 metadata、JOC index/object-count/EMDF 和 official-demo structure match 均保持 `NOT_RUN`；DEE parity 仍是独立待验证项。
- 4 个 Dolby Encoding Engine 进程与 GUI 在运行期间保持原样；PID 5012 当前未发现。
- 5.1 producer 只读审计已完成：`prepare_source_frame_from_proposals` → `prepare_selected_source_frame_candidate` → `search_selected_main_q` → `build_selected_frame_trial` 返回完整 E-AC-3 `frame_bytes`；candidate 只公开整帧 bytes、mantissa bit counts 和 EMDF cursor，没有六个 audio-block spans。`write_frame_body` 把 E-AC-3 block syntax 与 mantissas写入同一个 cursor。I0 assembler需要六个 caller-owned AC-3 audblk spans，因此目前没有安全的 typed adapter。后续小项需等 Wash 给出 I0 AC-3 audio-block syntax/producer 与 same-source lineage contract；现有 5.1 路径保持不变。D0 role join、downmix mode/phase、runtime source-generation owner join、native D0 algorithm/writer owner及 all-AU EMDF fit仍开放。编码运行需单独 preflight、Wash timeline registration和Fresh GO。
- D0 structural seam audit: the approved chanmap U16 pair flows through the D0 BSI segment and typed BSI composer; the D0 frame assembler calls that composer, places one block-2 skip-field at the layout span, and generates CRC2. No contract-complete D0 field/structure seam remains to integrate from the current packages. `D0FrameBitInput` still accepts caller-owned opaque audio spans and `CallerOwnedUnboundD0Block2InsertionPoint`; the next Wash contract should bind those typed spans and the insertion offset to the same source/generation owner before any new adapter is added.
- Windows 默认 `target/debug` 的 `selected_profile_gate` PDB 出现 LNK1285；同一 suite 在 `CARGO_TARGET_DIR=target/flat7x-validation`、`CARGO_INCREMENTAL=0` 下通过。后续 linker 回归优先使用隔离 target 目录。

## 2026-11-12 I0 AC-3 block-zero prototype

- Added `codec::ac3_audio_block::produce_ac3_audio_block`, a production-disabled single block-zero producer for explicit L/C/R/Ls/Rs/LFE PCM windows. It uses the ETSI long-transform formula with the existing hash-validated mathematical window provider, an explicit decoder exponent plan, the existing public 48 kHz BAP row calculator, and an AC-3 reconstruction-table quantizer with caller-selected `NearestTiesEven` policy.
- The supported structural row is `acmod=7`, `lfeon=1`, long blocks, chbwcod 48/endmant 217, seven LFE bins, caller-selected independent `cplinu=0`, explicit absent delta allocation, optional caller skip-field, and caller-supplied gainrng code 0. `lfeexpstr=1` follows the block-zero syntax rule. Source/callback generation, six-block assembly, whole-frame bit-budget fitting, I0 parent installation and product publication remain open.
- Added `validation::ac3_audio_block_parser` to parse the same supported block-zero branch, validate grouped/individual mantissa code spaces and require exact bit cursor consumption. Synthetic vectors exercise the scalar transform formula, published ETSI window landmarks, quantizer tie behavior, BAP 1/2/4 groups across all four full-band boundaries plus the full-band/LFE boundary, optional skip-field parsing and block controls.
- Verification: domain AC-3 tests 5 passed; validation parser tests 4 passed; selected 5.1 frame candidate regression 1 passed; prepared-input 5.1 rate/profile regression 1 passed; `cargo fmt --all -- --check`, workspace `cargo check --workspace --locked`, generated-map check and boundary check passed (91 scopes / 104 IDs / 0 unmaterialized roots / 6 partial roots). Strict workspace Clippy remains blocked by 9 existing diagnostics in analysis, DRC and JOC modules; no diagnostic references the new AC-3 files. Full workspace tests, W04, decoder acceptance, real PCM and output-file generation were not run.
- Default Windows link testing reported LNK1285 in pre-existing debug PDBs. Targeted runs passed serially in `C:\Users\chyinan\AppData\Local\Temp\jocforge-ac3-block-target`; existing repository target artifacts were left intact.
- Next I0 increment: extend the caller-plan block prototype to the six-block frame while preserving explicit controls, then obtain a same-source/generation receipt seam before connecting it to the parent. D0 audio and pair publication remain separate gates.

## 2026-11-13 typed pair staging adapter

- Added a crate-local `ProgrammeOutputStaging::append_flat7x_au` seam for domain-issued, attempt-bound `Flat7xAuAppendProposal` values. Preflight checks programme/attempt binding, matching I0/D0 identities, F13 consumer transaction, target frame/AU extents, sequential AU offset, and both CRC residuals before staging opens.
- The method delegates to the existing `AtomicOutputStaging::append_flat7x_au` adapter and has no production CLI caller. The route remains `Flat7xPairNotReady`; no production authorization or source producer was added.
- Three focused staging tests pass using an in-memory cursor: attempt mismatch, programme mismatch, and matched synthetic pair with nondurable completion rejection. The existing F15 receipt handoff integration test and I/O memory-sink test pass; selected 5.1 all-rate regression passes.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, generated-map check, and boundary check pass (92 scoped tasks, 104 IDs, 0 unmaterialized roots, 6 partial roots). Strict Clippy still reports existing diagnostics in domain and CLI code, including the existing source-frame output helper; no diagnostic points to the new pair staging checks. No staging file, `.ec3`, encoder, or DEE run was used.

## 2026-11-13 Flat7x source route and producer blocker

- The explicit Flat7x request now enters the existing worker/prepass/source pipeline with the D0 1,024-kbps rate row. At `stage_source_frame`, the path reaches the exact `NoApprovedAudioProductionRule` input and reports `Flat7xD0AudioProducerUnavailable` for the current source instead of stopping in profile resolution.
- Added `Flat7xD0CarrierPcmRows` with replaceable `PROVISIONAL_D0_FRONT3_PLUS_CHANMAP4_V1` ordering. It only selects borrowed PCM rows; it does not transform or encode D0 audio.
- `Flat7xD0AudioSpanReceipt::from_source_bound_producer` now exposes the typed producer boundary. The pair output path accepts a source-bound I0 frame, an opaque caller-owned D0 frame span, an explicit JOCForge `Flat7xSourceGeneration`, and F13/I0/D0 receipts, then validates F15 and reaches the existing atomic staging adapter.
- Core blocker: `produce_i0_audio_block_series` is fixed to five full-band channels plus LFE; the target dependent D0 needs a seven-carrier, no-LFE E-AC-3 audio-block writer. No D0 audio/BSI/EMDF bytes are synthesized. A producer for the typed row input and an F12 integrity receipt is the next replaceable seam.
- 5.1 single-frame output resolution and its six-rate regression remain unchanged. No encoder/DEE run or `.ec3` creation occurred.

## 2026-11-13 Flat7x source route to D0 block producer

- The CLI resolver now sends explicit Flat7x/1024 requests into the existing worker instead of returning at profile admission. Source execution reaches its first D0 span input and returns `Flat7xD0AudioProducerUnavailable { source_index, state: NoApprovedAudioProductionRule }` when no producer supplies spans. The handler preserves the named state and stages no partial AU.
- `Flat7xD0CarrierPcmRows` implements the provisional replaceable model `PROVISIONAL_D0_FRONT3_PLUS_CHANMAP4_V1`: I0 front L/C/R plus the four existing D0 chanmap rows. It checks equal nonempty extents and only selects borrowed row views.
- `Flat7xD0AudioSpanReceipt::from_source_bound_producer` is the exact D0 span interface. A receipt binds caller-owned block spans to the app's current pair source identity. The pair proposal adapter now accepts an explicit `Flat7xSourceGeneration`, same-source F13/I0/D0 receipts, an I0 full-frame candidate, and a caller-owned opaque D0 frame; it returns the bounded F15 pair proposal. `ProgrammeOutputStaging::append_flat7x_pair` connects that proposal to the existing atomic sink.
- The first unresolved audio structure is the seven-carrier D0 audblk writer. The local I0 producer's five full-band plus LFE layout cannot encode the selected D0 `acmod=5/lfeon=0/chanmap=0x1A00` row without a separate writer contract. The source rows remain provisional and no D0 audio, BSI, or EMDF bytes are generated in this increment.
- Focused I0/D0 proposal, D0 row mapping, named CLI producer error, CLI admission-route, staging Cursor, F15 handoff, I/O durability, and 5.1 six-rate tests pass. Workspace check, fmt, generated-map and boundary checks pass. Strict Clippy remains blocked by pre-existing diagnostics. No DEE/encoder run or `.ec3` creation occurred.

## 2026-11-13 correction: D0 coded channels and JOC carriers

The Wash-approved `JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1/PROFILE-ROW.json` distinguishes four D0 coded channels (`Ls`, `Rs`, `Lrs`, `Rrs`) selected by `acmod=5`, `lfeon=0`, and `chanmap=0x1A00` from seven JOC reconstruction carriers. This supersedes the earlier provisional `PROVISIONAL_D0_FRONT3_PLUS_CHANMAP4_V1` view, which mixed I0 front rows into the D0 audio input.

The implementation now passes only the four D0 coded-channel rows from the existing replaceable ADM role projection. The ADM-row-to-D0-channel relation remains provisional; JOC's seven-carrier reconstruction profile remains separate. The D0 E-AC-3 audio-block writer, source-bound producer receipt, pair parent commit, output exposure, and actual output remain open.

The approved U4/U7 writer boundary is now locally materialized as `codec::exponent_packing::write_eac3_exponent_row`. It preflights and writes a caller-supplied U4 seed followed by caller-supplied U7 groups, returning the exact `4+7*N` cursor receipt. The helper selects neither exponent values nor group count and carries no D0 lane/source identity. Full D0 block syntax, BAP/mantissa production, six-block overlap state, and pair/output admission remain open.

## 2026-11-14 D0 frame-tail policy seam

- Added `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits` to the source-bound D0 receipt and pair candidate. The production AU composer passes this typed policy to the D0 frame assembler, which writes `auxdatae=0`, explicitly zero-fills the remaining frame-level auxbits, and records the tail policy, span, and padding count before generating final CRC2.
- The existing frame assembler continues to enforce the only complete EMDF skip-field at zero-based block 2. Arbitrary opaque tail copying is gated behind domain `test-support`; its synthetic `0x96` vector remains a span/copy test only.
- Verification: default D0 frame tests 5 passed; `test-support` frame tests 7 passed; source-bound pair/AU integration 1 passed; selected 5.1 all-rate regression 1 passed; domain/application `cargo check --locked` passed; generated-map and boundary checks passed (92 scopes / 104 IDs / 0 unmaterialized roots / 6 partial roots). Strict Clippy reports 11 existing issues outside the changed D0 seam.
- Remaining: implement the four-channel 253-bin D0 BAP/exponent/mantissa audio-block producer with six-block state, bind F13/F12 receipts and source generation into the CLI and parent commit, then validate full AU/output. Do not run an encoder or create `.ec3` before Wash Fresh GO.

## 2026-11-14 D0 BAP provider and typed rows

- Added `codec::bap::flat7x_d0_provider` as a D0-only profile type. Its tables are independently materialized from ETSI TS 102 366 V1.4.1 Tables 6.6–6.12 and 6.14–6.16; table-set SHA-256 `5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe`.
- `allocate_flat7x_d0_bap_row` consumes a typed 253-bin decoded-exponent row. `allocate_flat7x_d0_bap_frame` preserves six blocks × four coded channels × 253 bins with no LFE. Both reuse the existing pure BAP calculation core behind D0-specific input/output types; they do not serialize mantissas or audblk syntax. Existing selected 5.1 provider tables and APIs remain unchanged.
- Verification: ETSI table landmark/coverage test 1 passed, D0 row/frame suite 4 passed, existing `bap_converter_search` suite 14 passed, workspace `cargo check --locked` passed, and the 5.1 all-rate regression passed (1). Generated-map and boundary checks pass (92 scopes / 104 IDs / 0 unmaterialized roots / 6 partial roots). Strict domain Clippy reports 9 existing diagnostics in analysis/DRC/JOC, none in D0 BAP.
- Remaining: PCM→512-sample transform and D15 exponent producer/history, frame-wide SNR and bit-budget choice, mantissa serialization, complete source-bound D0 audblk/receipt, CLI integration, and parent F12/F13/F15 atomic AU commit. Real encoder execution remains Fresh-GO gated.

## 2026-11-14 D0 PCM spectrum and gainrng

- Added `codec::flat7x_d0_spectrum::transform_flat7x_d0_pcm_block` for four typed D0 coded-channel windows. It consumes prior/current 256-sample portions, calls the shared pure `transform::long_mdct::forward_long_mdct_512` core, returns four 253-bin coefficient rows, and derives per-channel `gainrng` from all 512 input samples with the public ETSI Table 6.33a thresholds. The existing 5.1 adapter calls the same core with the original operation order.
- The output is a nonpublishing spectrum candidate; it has no history installation, exponent coding, BAP join, mantissa serialization or audblk bytes. The existing 5.1 transform and route are unchanged.
- Verification: D0 spectrum 2 tests passed, D0 BAP 4 passed, existing BAP suite 14 passed, and the 5.1 all-rate regression 1 passed; domain check passed. The remaining gate is D15 exponent extraction/preprocessing plus six-block history, then mantissas/audblk and source receipts.

## 2026-11-14 D0 D15 exponent rows

- Added `codec::flat7x_d0_exponents::prepare_flat7x_d0_d15_exponent_row` and `write_flat7x_d0_d15_exponent_row`: each typed D0 row returns 253 raw/decoder-visible exponents, a U4 seed, 84 U7 groups and a 592-bit shared-writer receipt.
- ETSI defines raw leading-zero exponents, U4 seed range and legal D15 differential range but leaves the preprocessing algorithm open. Wash confirms no approved unique algorithm and recommends the lower-only two-pass projection; JOCForge names it `JOCFORGE_D15_LOWER_ONLY_SEED15_SLEW2_V1`. It caps the seed, caps rises to previous+2 and backward-lowers earlier exponents to next+2, keeping decoded exponents at or below raw. It is a replaceable local model policy, not a DEE claim; production remains disabled.
- Verification: D15 suite 6 passed, including four spike vectors, 10,000 deterministic invariant rows, and U4/U7 serialization; D0 spectrum 2, D0 BAP 4, existing BAP 14, workspace check and 5.1 all-rate regression 1 passed. Clippy continues to show 9 unrelated diagnostics.
- Remaining: normalize/quantize mantissas with the decoder-visible row, assemble full six-block D0 `audblk` syntax, fit the frame-wide bit budget, produce F12/F13 receipts, then connect the CLI and parent F15 atomic commit.

## 2026-11-14 D0 mantissa packing

- Added `codec::flat7x_d0_mantissa::pack_flat7x_d0_mantissa_block`, which pairs each of 253 spectrum coefficients per D0 coded channel with that channel's decoder-visible D15 exponent and the selected block's BAP row. It preserves `[Ls, Rs, Lrs, Rrs]`, applies the existing `quantize_mantissa`, and reuses `grouped_quantizer::pack_block_mantissas` for BAP 1/2/4 grouping and MSB-first bytes.
- The selection is named `JOCFORGE_EXISTING_EAC3_QUANTIZER_V1` and remains replaceable with `production_allowed=false`. Wash package `CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1` is input-only and contains no expected quantizer/group codes; local tests assert wrapper consistency against the shared quantizer, not independent codec conformance.
- Two focused tests pass. The stage remains block-local: frame-wide bit-budget/BAP search, complete D0 `audblk`, six-block history/owner, source-bound F12 receipt, CLI production caller and parent F15 commit remain open. The 5.1 all-rate regression remains required after this shared grouping extension.


## 2026-11-14 D0 audio-block and frame candidate

- Added the 23-bit typed D0 `audfrm` control prefix and the four-channel D0 `audblk` writer. It consumes spectrum, D15 and BAP rows; writes dynamic-range presence, dither, coupling/SPX state, four D15 rows, `gainrng`, and mantissas; and returns block two as pre-skip/post-skip spans. The local block policy uses `chbwcod=60`, `frmfgaincode=0` (default fast gain 4), and default BAP parameters.
- `produce_flat7x_d0_audio_frame` evaluates raw q=1023..1, recomputes all six-block BAP and exact grouped mantissa bit costs, skips the q=0 zero-BAP bypass and selects maximum fitting content bits; ties keep the higher raw q. It makes no q-cost monotonicity assumption. The local policy is `JOCFORGE_FLAT7X_D0_MAX_ALLOCATED_BITS_Q_HIGH_TIE_V1`.
- The selected candidate is assembled as a complete 4096-byte in-memory D0 frame with one block-two EMDF skip-field, zero aux-tail, and CRC2 receipt. Synthetic frame tests 3 passed; assembler 5, layout 4, D0 unit filter 11, application pair-source 4, and selected 5.1 all-rate regression 1 passed. `production_allowed=false` and no source-history or F12/F13/F15 receipt is issued.
- Remaining: create six spectra from callback PCM with a programme/attempt/source-generation history owner, bind the provisional role map and same-source DRC/F13 receipts, issue D0 F12 integrity, connect the CLI and parent F15 commit, then validate real input/output. Encoder execution remains Fresh-GO gated.

## 2026-11-14 D0 source PCM block-series candidate

- Added `codec::flat7x_d0_pcm_source::stage_flat7x_d0_pcm_block_series` and `produce_flat7x_d0_frame_from_pcm_source`: four `[Ls, Rs, Lrs, Rrs]` source rows flow through six transform/spectrum/D15 blocks and the existing BAP/q-search/audblk/frame composer. The complete 4,096-byte frame and next overlap history are returned together as candidates.
- Added application seam `produce_flat7x_pair_d0_pcm_frame_candidate`: it consumes the exact D0 rows from `Flat7xPairSourceBinding` and carries that binding's source identity and role-map receipt beside the frame/history candidate.
- PCM-series unit tests (3), the D0 frame integration suite (4), format check, and workspace `cargo check --locked` passed. Full package test linking hit stale PDB failures in unrelated integration binaries. Strict domain Clippy still reports 9 existing errors outside the new module.
- Verification also includes the application source-binding test (1 passed). Remaining: programme/attempt/source-generation history owner; policy for suppressed sources 0/1 and F15-only commit; valid/padded sample lineage; same-source I0/D0/F13/F12 receipts; CLI pair caller and durable final-output lifecycle. `production_allowed=false`; no file output was created.

## 2026-11-14 role-specific Flat7x F12 pair candidate

- Added `Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024` and `IntegrityProgrammeOwner::stage_flat7x_i0_d0_f12_pair_for_callback`. It validates I0 at 640 kbps and D0 at 1,024 kbps, issues I0 `AbsentPublishedPayloadV1` and D0 protected-EMDF receipts, and returns both with one shared before/after Integrity generation and one owner receipt.
- The focused pair-receipt test passed (1), including 2,560/4,096-byte frame extents, D0 cursor binding, zero CRC16 residuals, and unchanged committed owner before parent installation.
- Remaining: bind both receipts to the actual F13 metadata proposal and pair source at the application/CLI seam; provide the rate/extent receipt at the product callback; connect pair F15 staging and install the one Integrity/D0 history proposal only after durable append. Production remains false.

## 2026-11-14 Wash source-map package

- Registered `JOCFORGE-MAIN-ISOLATED-7.1-SOURCE-MAP-WASH-V1`, manifest `1c66063cd651daa8c7b0fc1b205db4d9521a54ca33b645e711ebd13989a3dd58`, seal `e799eebf51702a863212fb2f5a27b9ad2c773768e384ecdc2b95cb844f8f4d81`.
- Normal/optimized package-only checkers passed and 18 isolated source snapshot hashes matched HEAD. The mapped reuse scope is role separation and I0→D0 pair order only; legacy main audio/rate paths are excluded. The production CLI pair caller remains absent.

## 2026-11-14 source-bound pair F15 candidate

- Added `stage_flat7x_pcm_pair_for_f15`, which takes source-bound I0/D0 frame candidates, one Integrity F12 pair proposal, F13 metadata, explicit source generation and the F15 output owner. It returns one whole-AU append proposal plus D0 after-history and retains access to the matching Integrity after-root/owner receipt; all state remains pending for the caller's durable append transaction.
- Focused application integration passed (1): exact 2,560+4,096 extents, ordered 6,656-byte AU, F13 identity, F12 roles and carried D0 history. No filesystem write or parent root install was performed.
- Remaining: replace test-fixture F13 inputs with the production metadata proposal; bind source PCM interval/generation/tail extent; connect the CLI callback and durable I/O receipt to one parent swap.

## 2026-11-14 source-bound F13/F14K-to-F12 bridge

- Added `application::flat7x_pair_integrity::stage_flat7x_pair_f12_integrity`: it checks the I0/D0 source-role-map binding, the F13 source/consumer identity and the F14K protected payload lineage, derives the D0 EMDF payload cursor from its frame receipt, and stages the fixed-role-rate F12 pair proposal.
- The F15 integration test drives a source-bound PCM→D0 frame, a source-bound I0 frame, F13/F14K test data, the pair F12 proposal and one in-memory F15 AU append candidate. It passed (1).
- Production caller, callback PCM extent/EOF tail receipts, parent durable append, and atomic Integrity/history installation remain open; production is disabled.

## 2026-11-14 reader-issued D0 PCM source extent

- Added `Flat7xD0PcmSourceExtent` from the actual `FrontendPcmCallbackFrame`: it carries `CodecSourceIndex`, `FrontendCallbackIndex`, `AdmSampleIndex first_sample`, valid samples, and padding samples. The constructor checks `source_for_callback`, `first_sample = callback × 1536`, and the 1536-sample extent.
- The exact receipt now flows through `Flat7xPcmSourceRows`, the source PCM projection, `Flat7xPairSourceBinding`, and the D0 PCM frame candidate. D0 frame staging verifies the bound extent and requires zero words in padded row tails.
- Cached drain rows carry no source-specific reader extent; Flat7x source staging stops before candidate production for that delivery. This leaves the cached-drain source recipe visibly gated.
- `cargo fmt --all -- --check`, workspace `cargo check --locked`, domain/application/CLI test-target compile checks, generated-map check, and boundary check passed. No test binary or encoder was run. Strict workspace Clippy reports 18 errors and 1 warning, all outside the files changed in this slice.
- Remaining: Wash-backed per-source cached-drain rows and terminal extent policy; D0 history owner and suppressed-source installation; production D0 settings/receipt caller; one parent F15 durable append and root swap. `production_allowed=false`.


## 2026-11-14 CLI D0 history and frame candidate

- Extended `CodecOwnerRoot` with an optional Flat7x D0 overlap history. The Flat7x cold value is zeroed 4×256 F32 bit words, canonical state includes lowercase hex for all words, and programme reset restores the same cold history. The ordinary 5.1 Codec root serialization stays byte-identical.
- Suppressed sources 0/1 now stage six-block D0 PCM history from their source-bound rows and install it through the existing callback parent. Cached drains use the explicit provisional policy `CachedDrainRepeatPreviousCallbackV1`, which reuses previous cached PCM bits with a logical source extent.
- Published sources >=2 now call `produce_flat7x_pair_d0_pcm_frame_candidate` using the committed history, F14K protected container and replaceable local settings `JOCFORGE_FLAT7X_D0_FRAME_DEFAULTS_V1` (DrcOwner dialnorm with fallback 31, `compr=1`, no mixing/addbsi, no per-block dynamic-range code). The candidate is a complete D0 frame; the CLI currently stops before paired I0/F12/F15 output.
- Verification: workspace `cargo check --locked` and `cargo check -p jocforge --tests --locked` passed. No tests ran. Boundary/map checks passed after this slice. Strict Clippy reports 18 pre-existing errors and 1 warning, with no diagnostic in this slice.
- Next: produce the I0 AC-3 frame from the same source binding, join real F13/F14K with the paired F12 integrity proposal, route the 6,656-byte pair through F15 and the parent commit, then finalize the Flat7x stream.
