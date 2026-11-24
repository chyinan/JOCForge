---
status: partial-implementation-locally-verified
topic: implementation-agent-status
spec: JOCForge
confidence: repository-status
last_updated: 2026-11-23
tags:
  - implementation
  - handoff
  - 7.1-flat7x
---

# 实现 Agent 状态

## 当前状态

- 2026-11-23 整曲问题定案（owner 授权自行决定）：**不做分段编码、不做流式 PCM 改造**，改为显式输入预算。理由：分段编码会改变输出（dialnorm 是全节目测量、AU 连续性与比特分配都跨段），直接毁掉刚建立的逐字节一致性；流式 PCM 要重写 domain 的整份驻留假设（`Arc<[u8]>`、哈希、ADM payload），成本远超收益且同样改变契约。落地：wasm 桥接新增 `MAX_RESIDENT_INPUT_BYTES = 512 MiB`，`push_bytes` 一旦超预算立即返回 `InvalidInput` 并附可读 detail，避免编码中途 OOM 把模块打废（wasm 陷阱是不可恢复的）。`docs/architecture/wasm-bridge.md` 记录该预算与取舍。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`（0 告警）、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all --check`、`scripts/check-boundaries.py`（91/104/0/6）通过；`test-wasm.mjs` → `wasm abi: PASS`；`compare-wasm-encode.mjs` → `bit_exact=true`。剩余不确定：receipt 语义仍沿用 domain 唯一构造函数（决定不改契约，理由见 wasm-bridge.md）。

- 2026-11-23 桥接收敛为薄壳 + 单份输入：`jocforge-wasm/src/lib.rs` 删掉了自带的整段输入准备（parse → `prepare_selected_job_input` → `bind_primary_control_rows`，`let _ = prepared;` 后丢弃），改为直接调用管线入口并只做错误分类——此前浏览器每次编码会解析并准备 ADM **两遍**，且存在两条准备路径漂移的风险。新增 `encode_adm_resource(Arc<[u8]>)`（wasm-only），`Encoder::finish` 用 `mem::take` 把已积累的输入交给它，输入只驻留一份。错误分类按 `EncodeRequestError` 变体分到 `InvalidInput`/`Encode` 两类，ABI 冒烟测试的 `error_category()==0` 契约保持。新增 `docs/architecture/wasm-bridge.md` 记录 ABI、内存模型、`memory://` 例外、栈参数、执行模型、验证命令与已知限制。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`（0 告警）、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿，wasm crate 5 项）、`cargo fmt --all`、`scripts/check-boundaries.py`（91/104/0/6）通过；重建 release wasm 后 `compare-wasm-encode.mjs` 仍 `bit_exact=true`，`test-wasm.mjs` 仍 `wasm abi: PASS`。剩余不确定：整曲浏览器编码未实测（4 GB 地址空间 vs 单份 PCM）；`admission_rejection_detail` 诊断仍留在代码里。

- 2026-11-23 浏览器侧改为真正的拉取式流，逐字节一致性保持不变：`SpatialFrameProducer::run` 的消费端改用 `&mut dyn FnMut(u64) -> SpatialFrameResult`——native 仍是单槽 `sync_channel(1)` + 生产者线程（行为未变），wasm32 由消费者按序拉取，**不再预物化全部帧**，内存只随单帧增长。`run_inner` 的 callback 循环相应改为 `frames(callback_value)?`。临时的目的地探针（`destination=/absolute=/parent=/file=`）已从错误 detail 里删除，只保留 typed `admission=...`。度量（同一 revision）：release wasm 重建后 `compare-wasm-encode.mjs` → `wasm push=0 finish=1 / excerpt=2468488 wasm=32768 native=32768 bit_exact=true`；`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all --check`、`scripts/check-boundaries.py`（91/104/0/6）全部通过。剩余不确定：整曲（1.7 GB，52 声道）浏览器编码仍未实测——现在受限于 4 GB 地址空间与 PCM 单份驻留，而不是帧缓冲；`is_absolute` 在 wasm 恒假仍只有行为观测。下一步：评估整曲浏览器编码的可行边界（分段编码 / File System Access），或把 `MemoryOutputStaging` 的 receipt 语义（`atomic_replace`/`parent_fsync`）与 owner 对齐。

- 2026-11-23 浏览器与原生产出**逐字节相同**（首次取得该证据）：release wasm 5,682,265 B，Node 驱动同一段 2,468,488 B / 12288 帧 ADM excerpt，`push=0 finish=1`，两端均 32,768 B（8 AU），`bit_exact=true`；`scripts/test-wasm.mjs` 同一产物 `wasm abi: PASS`。为此落地四处改动：(1) F15 目的地接受保留 scheme `memory://<product>`（`MEMORY_DESTINATION_SCHEME`，仅该标记可跳过 `is_absolute`，其余 owner provider、profile 与身份校验照跑，native 路径仍强制绝对路径）；(2) `ProgrammeOutputStaging::from_admitted_plan` 识别该标记并只保留 typed binding；(3) meter prepass 与 spatial frame producer 在 wasm32 内联执行（后者一次性物化全部帧，短片段可用、整曲需要流式端口，已在注释与 handoff 标注）；(4) `.cargo/config.toml` 给 `wasm32-unknown-unknown` 加 `-zstack-size=16777216`，此前 `prepare_spatial_production_core` 因默认栈过小触发 out-of-bounds。另修 `ProviderTree` 内嵌查表：build script 记录的是宿主分隔符（`\`），查表时先归一化为 `/`。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`（0 告警）、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all`、`scripts/check-boundaries.py`（91/104/0/6）全部通过。剩余不确定：整曲（1.7 GB / 52 声道）在浏览器里仍受「帧全量物化 + 4 GB 地址空间」限制，未测；`is_absolute` 在 wasm 恒假只做了行为观测，未查 std 实现。下一步：把 spatial producer 换成真正的流式端口（pull 模型），再评估整曲浏览器编码。

- 2026-11-23 wasm 逐字节对照定位到根因：`wasm32-unknown-unknown` 上 `Path::is_absolute()` 对 `/jocforge-browser/output.ec3` 返回 **false**（同一次探针里 `parent()` 仍解析为 `Some("/jocforge-browser")`），于是 `admit_selected_f15_plan_for_profile` 以 `InvalidFinalDestinationIdentity` 拒绝；换成 `C:\jocforge-browser\...` 反而更早在 `file_reference_parts` 处以 `InvalidPath` 失败（无父目录）。结论：F15 destination 契约要求绝对命名空间路径，而浏览器隔离区在这套 std 下**无法构造绝对路径**，因此该门在 wasm 上不可满足，需要契约层决定——建议加一个显式的无命名空间标记（例如 `memory://<product>`），让准入在标记存在时跳过 `is_absolute` 检查，native 保持严格。诊断代码已在位：`jocforge_application::output_publication::admission_rejection_detail`（wasm-only）+ pipeline 的错误 detail 会带出 `admission=...`、`absolute=`、`parent=`、`file=`。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`（0 告警）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all`、`scripts/check-boundaries.py`（91/104/0/6）通过；release wasm 5,682,265 B 可在 Node 实例化并跑到 F15 准入。剩余不确定：`is_absolute` 恒假的成因（`std::path` 在该目标的实现）只做了行为观测，未读 std 源码确认。下一步：按 owner 决定实现无命名空间标记，然后重跑 `scripts/compare-wasm-encode.mjs` 取得字节相等证据。另注：本轮一次自动化编辑曾损坏 `encode_request.rs`（已由子代理按 index 基线 + 编辑清单重建，2955 行，双目标零告警）。

- 2026-11-23 产品编码入口不再是空壳：`jocforge_application::encode` 改为「校验并返回已准入计划」——签名变成 `encode(prepared, resolved_profile, attempt_id, staging_owner_id, final_destination) -> Result<AdmittedProductionPlan, EncodeError>`，内部走 `admit_selected_f15_plan_for_profile` 并复核 `is_bound_to`；管线侧改调该入口，`EncodeError::SourcePipelineUnavailable` 不再被构造（枚举变体暂留以兼容公开契约）。审计脚本两处随之更新：`'SourcePipelineUnavailable' in app` → `'pub fn encode(' + 'AdmittedProductionPlan'`；R4 条款改为要求 encode body 含 `admit_selected_f15_plan_for_profile(`；`AGENTS.md` 也改成如实描述。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all`、`scripts/check-boundaries.py`（91/104/0/6）均通过。

- 2026-11-23 浏览器逐字节对照已推进到 F15 准入：`short_adm_excerpt` 落盘（2,468,488 B，12288 帧）+ native 码流（32,768 B，8 AU，sha256 `30581464dc…`）由新增测试 `dumps_excerpt_and_native_bitstream_for_cross_target_parity` 产出；`scripts/compare-wasm-encode.mjs` 驱动 release wasm（5,682,265 B）跑同一段字节。沿途修掉三个 wasm 运行期阻断：`SystemTime::now()`（改计数器）、`std::process::id()`（改固定 0）、`ProgressBar` 的 `Instant`/`IsTerminal`（wasm 下置为空实现）。当前 wasm 运行停在 `admit_selected_f15_plan_for_profile` 的拒绝，错误被 `EncodeError::F15AdmissionRejected` 压平成一句话，尚不知道具体因子（destination 身份 / owner provider / profile 维度三者之一）。剩余不确定：该拒绝原因未定位；native 与 wasm 仍未取得字节相等证据。下一步：给 wasm 分支加一次性诊断（把 `F15PlanAdmissionError` 的 Debug 放进 detail），定位后修因再重跑 `compare-wasm-encode.mjs`。

- 2026-11-23 浏览器编码入口接通（未做 wasm 运行时验证）：`run_inner` 增加 `resource: Option<Arc<[u8]>>` 参数——给了字节就跳过 `normalize_existing_input` 与 `read_resource(path)`，路径只作为 job XML 名义引用；返回值改为 `RunProduct::{Published, Bytes}`，wasm32 用 `ProgrammeOutputStaging::into_product_bytes()` 交出内存产物；结尾的 `std::fs::metadata` 长度复核按目标 cfg（wasm 端没有第二份可 stat 的副本，receipt 就是从 buffer 铸的）；`run_worker_with_object_count` 在 wasm32 直接内联调用 `run_inner`，不再依赖 `thread::scope`（原先 spawn 在浏览器必失败）。`encode_adm_bytes` 的 wasm32 分支不再返回 stub，改为走真实管线并返回码流字节。度量：`cargo check --workspace --all-targets --locked`、`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked`（0 告警）、`cargo clippy --workspace --all-targets --locked`（无 error）、`cargo test --workspace --locked`（全绿）、`cargo fmt --all --check`、`scripts/check-boundaries.py`（91/104/0/6）均通过。审计脚本同步一处字面量：flat7x 实验候选分支的 `return Ok(())` 改为 `return Ok(RunProduct::Published)`（语义不变：该分支仍然不 finalize、不安装 DONE）。剩余不确定：wasm 产物尚未在 Node 里跑真实 ADM 编码，因此 native 与 wasm 的逐字节对照仍未发生；浏览器侧 `thread::scope` 之外的栈预算（`ENCODE_REQUEST_STACK_BYTES`）在 wasm 上无对应机制。下一步：用 short ADM excerpt 走 `node scripts/test-wasm.mjs` 式驱动，比较 native 与 wasm 字节。

- 2026-11-23 内存侧适配层落地（浏览器编码的前置）：新增 `jocforge_io::provider::ProviderTree`（目录 / 内嵌字节两种实现，`read(relative)` 统一入口，`child(prefix)` 取子 bundle），6 个 provider loader 全部改为接收 `ProviderTree`，`package_file`/`read_file` 双跳读取合并为一次 `tree.read`，证据路径统一走 `tree.display`；新增 `jocforge_io::atomic_output::MemoryOutputStaging`，按与原子文件适配器相同的协议（身份、长度、精确回读、frame SHA、CRC 残差、guarded receipt）在内存 buffer 上追加并 finalize；`ProgrammeOutputStaging` 的核心类型按目标切换（native=文件，wasm32=内存），`unpack_provider_supplements` 在 wasm32 改为返回内嵌 `ProviderTree` 而非报错。度量：`cargo check --workspace --all-targets --locked` 通过；`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked` 通过且 0 告警；`cargo clippy --workspace --all-targets --locked` 无 error、改动文件无新告警；`cargo test --workspace --locked` 全绿（含 `native_rate_job` 384 kbps 真实编码用例）；`scripts/check-boundaries.py` 通过（91/104/0/6）。过程中修掉两处真实缺陷：`ProviderTree` 原生读把 canonicalize 结果与未规范化 root 比较（Windows `\\?\` 前缀导致全部读失败）；`SelectedSpatialColdProviderRoots` 去重只比 root 导致 `child()` 出来的同根子包被误判相同。剩余缺口：`encode_adm_bytes` 的 wasm32 分支仍是 stub——`run_inner` 仍以输入路径读取 RIFF 并在结尾用 `std::fs::metadata` 复核长度，浏览器端要真正出字节，下一步得让 `run_inner` 接受调用方字节并返回内存产物。未做浏览器运行时验证（无 wasm 产物在 Node 里跑）。

- 2026-11-23 架构拆分：新增库 crate `crates/jocforge-pipeline`，把 encode 管线 13 个模块（`encode_request`、`source_execution`、`output_publication`、`programme_roots`、`meter_prepass`、`loudness_cache`、`selected_*`、`cmaf`、`progress` 等）与 provider 资产嵌入用的 `build.rs` 从 `jocforge-cli` 迁出。`jocforge-cli` 只保留 `main.rs`（进程入口）与同名 `lib.rs` facade 重导出，其 7 个集成测试零改动。`jocforge-wasm` 改为依赖 `jocforge-pipeline`，不再依赖 `jocforge`（CLI 包）。`DEPENDENCY-RULES.md`、`AGENTS.md`、`IMPLEMENTATION-MAP.json` 与 `check-boundaries.py`（依赖集合、`build.rs` 归属、unsafe 集、源码路径）同步更新。度量：`cargo check --workspace --all-targets --locked` 通过；`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked` 通过且无告警；`cargo clippy --workspace --all-targets --locked` 无 error（仅既有 unused 诊断）；`cargo test --workspace --locked` 全绿；`scripts/check-boundaries.py` 通过（91 scoped tasks / 104 IDs / 0 unmaterialized / 6 partial）。同批修掉上一轮遗留缺陷：`crates/jocforge-cli/src/main.rs` 的 `command::main` 是私有函数，native bin 实际无法编译（此前只跑 `-p jocforge-wasm` 库检查，未暴露）。剩余不确定：`docs/implementation/*.md` 视图用文本替换而非 `scripts/render-implementation-maps.py` 重渲染；CLI 侧 `test-support` 现转发到 pipeline，仅由全量测试佐证。下一步：把 850 行的 `run_inner` 拆成命名阶段，再让 wasm32 的 `encode_adm_bytes` 接到内存 publication owner。

- 2026-11-23 wasm32 入口已测通到准备阶段，尚未产生 EC3。`cargo check -p jocforge-wasm --target wasm32-unknown-unknown --locked --offline` 通过，无告警。`scripts/check-boundaries.py` 通过（91 scoped tasks / 104 IDs / 0 unmaterialized / 6 partial）。`JOCFORGE_ADM_PARITY` 指向本机《苍白的骑行 ADM》时，`jocforge-wasm` 4 项测试通过：RIFF 为 48 kHz、52 声道、block align 156；`prepare_selected_job_input` 与 `bind_primary_control_rows` 成功；`encode_adm` 与 bridge `finish` 同为 `FailureCategory::Encode`，detail 为 `source execution unavailable: selected callback and AU publication path are not wired`。domain `dsp_workers::tests` 1 项通过。源码不写本机绝对路径，全文件不进仓库。剩余不确定：native 与 wasm 的码流逐字节对照尚未发生，因为内存 publication owner 未授权，产品 `encode` 仍停在该门。下一步：只在单独授权后把短切片接到同一内存编码出口，再比 native 与 wasm。

- 2026-11-19 CLI 根级 `--help` 的 JOCForge 标题已加入 OpenJOC 同款四段 24-bit RGB 渐变，说明副标题用绿色、帮助分区标题用 OpenJOC 的青色；交互终端着色条件及纯文本输出行为与 OpenJOC 一致。workspace version 已升到 1.0.0，Cargo.lock 同步更新。

- 2026-11-19 CI no longer runs the source inventory/hash audit or promotes Clippy warnings with `-D warnings`. The prepared-input regression now binds WAVE channels to CHNA physical rows, rejects unsupported authored start/end times, and keeps 768 kbps accepted; F01E JSON fixture checkout line endings are pinned across platforms. CI retains workspace fmt/check/Clippy and targets prepared-input, CLI migration-boundary and dynamic 10/22/36/38-row mapping suites; broader domain fixture reconciliation remains separate.
- JOCForge root and `encode` help now follow OpenJOC's sectioned CLI layout and use the supplied logo. `encode -h` and `encode --help` are supported, with direct-file defaults and the Flat7x candidate-output requirement shown in the command help.
- Flat7x ID14 capacity fallback keeps seven-row objects in full-matrix mode, reduces their band count and preserves the seven carrier gains. The focused budget-retry test passes. A local full-song candidate rendered through OpenJOC 0.20.2 with the 26.8-second peak reduced from 0.623 to 0.042; full LAV/PotPlayer playback remains unverified.
- 2026-11-15 F15 receipt ledger now retains the full Codec-owned `Flat7xD0SourceGenerationReceipt` copied from D0 block-two placement. The application requires exact equality with the I0 candidate receipt; the domain binds source transaction, callback lineage, source index and dispatch to F13. The F15 `Flat7xSourceGeneration` remains a separate field/type. Focused receipt tests, workspace check, fmt and selected 5.1 all-rate routing pass. Clippy `-D warnings` remains red on existing diagnostics; production authorization remains disabled.
- Profile-aware ID14 header coverage now asserts the serialized U3 comes from the profile layout selector: Flat7x writes 1 while an input downmix value of 3 is supplied; selected 5.1 has no layout selector and preserves U3=3. The focused test and selected 5.1 all-rate regression pass; production code is unchanged.
- A test-only F13→F14K roundtrip now uses `MetadataOwner::stage_source` and its actual F13 serializer bytes, protects them through F14K, then parses with the receipt's Flat7x `[11,14]` sequence. It verifies exact payload order/bytes, whole-container extent, checksum ranges/codes and zero terminal padding. The ID11/OAMD and ID14/JOC values are synthetic metadata inputs; no audio is generated. Focused roundtrip and 5.1 all-rate route pass.
- E-1106 captured the same Love Song input on the default selected 5.1 route at 1024 kbps: 8,417 one-frame AUs, 12,928,512 coded samples and 34,476,032 bytes. OpenJOC 0.20.1 compatibility admitted every AU, and its 6-channel 48 kHz float32 render is available for A/B. This is an isolated comparison baseline; no listening conclusion is recorded.

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

## 2026-11-14 Flat7x BSI 与 candidate capture

- Love Song (`D:\Love Song 方大同.wav`) 是 36-channel / 48-kHz / PCM24 ADM BWF，含 `axml` 与 `chna`。用户直接授权的 Release attempt PID `16072` / attempt `1794647185411283100` 已完成，输出 `D:\JOCForge-attempts\love-song-flat7x-1664-release-20261114-170625\love-song-flat7x-1664.candidate.ec3`，8,417 AU / 56,023,552 bytes。F15 stage/readback/file sync/directory sync receipt SHA-256 为 `6101459cc6b04cda1b192cf7add7a8dc2e0775c8f4965b08e85040bea19e3b0c`。
- CLI D0 BSI settings 现在通过 `bind_flat7x_d0_bsi_source_inputs` 从同一 PreparedJobInput、SharedAnalysisReceipt 和 F06ND after-proposal 取出 dialnorm、RF compr、六项 line DRC codes 与 validated authored downmix；source identity 与 Flat7x role-map receipt 在 frame production 前再次比对。策略保持 `production_allowed=false`，optional information/addbsi 仍按明示 absence policy 留空。
- Flat7x source callback 完成后走专用 `.candidate.ec3` 捕获：检查完整 F15 pair ledger 与长度，使用 create-new candidate file、现有 stream stage helper、读回/hash/文件同步/目录同步；失败路径保留 candidate extent 并写 `.failure.txt`。generic finalizer 拒绝 Flat7x，候选路径不提交 Programme DONE root。
- 修复了 Flat7x suppressed source 0/1 的 F15 source-clock proposal、ID14 header parser 的 profile index 1/3 admission，以及 I0 gainrange passthrough / exponent-reuse syntax mismatch / long-block provisional policy。对应 targeted tests 与 CLI check passed；Release run 通过所有 8,421 callback 并 capture 完整 candidate。OpenJOC parse/decode/render、playback 与 W04 仍为 `NOT_RUN`；候选不与 DEE 等价，5.1 保持不变。
- Wash 元数据包 `love-song-flat7x-object-to-joc-row-metadata-only-wash-v1`（manifest `ec88b6cd216857d5fb758a892e0d8540a51d7fcea78f8d2566d68d6ac284dd30`；seal `188051a489ffcc03a8894c781208361a0664b64bbd639a00be27cf136b669812`）确认 26 个 ADM Object tracks 与 CHNA/AXML UID+pack 显式连接；到 target 15 JOC objects/rows 和七个 carriers 的 crosswalk 为 `UNBOUND`。候选可使用用户授权的版本化 provisional selection policy；不据此声明 object/spatial compliance 或 listening parity。
- D0 `frmsiz=2047` 是 4,096-byte target frame 的 caller-owned contract value；Wash `FLAT7X-D0-FRMSIZ-U16-MINUS-ONE-11BIT-V1` 绑定 U16-minus-one 和 11-bit writer，但 D0 state/value owner join 仍 `UNBOUND`。

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

- Added `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits` to the source-bound D0 receipt and pair candidate. The production AU composer passes this typed policy to the D0 frame assembler, which zero-fills `nauxbits`, writes `auxdatae=0` followed by `crcrsv=0`, and reports those receipt fields before generating final CRC2. `encinfo` remains in the BSI extension.
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
- Published sources >=2 now call `produce_flat7x_pair_d0_pcm_frame_candidate` using committed history, the F14K protected container and local D0 settings `JOCFORGE_FLAT7X_D0_FRAME_DEFAULTS_V1`. The CLI builds an I0 frame with `JOCFORGE_FLAT7X_I0_AUDIO_POLICY_V1`, joins actual F13/F14K with paired F12 receipts, and creates the first 6,656-byte F15 in-memory proposal with source-generation model `JOCFORGE_FLAT7X_SOURCE_GENERATION_SOURCE_INDEX_V1`. The callback then stops before durable output-owner persistence and parent installation.
- Verification: workspace `cargo check --locked` and `cargo check -p jocforge --tests --locked` passed. No tests ran. Boundary/map checks passed after this slice. Strict Clippy reports 18 pre-existing errors and 1 warning, with no diagnostic in this slice.
- Next: persist the pair-aware output owner across AU callbacks, stage the in-memory proposal through the durable 6,656-byte sink, and install Integrity/Codec/Output roots in one parent replacement before finalization.

## 2026-11-14 Flat7x aggregate F15 output configuration

- Added profile-aware F15 admission for the CLI-resolved Flat7x layout. The output projection uses the pair target aggregate of 1,664 kbps and 6,656 bytes per AU; OutputConfiguration accepts the extent and ProgrammeOwnerRoots READY checks the same value. D0's selected carrier rate remains 1,024 kbps, and the 5.1 output profile remains unchanged.
- This prepares the F15 output root for the in-memory pair candidate. The source callback still stops before durable output-owner state, the pair append sink, and the parent swap.
- Workspace check, CLI test-target compile, generated-map check, and boundary check passed. Strict Clippy reports 18 existing errors and 1 warning outside the changed files. No test binary or encoder was run.
- Next: persist the pair ledger across AU callbacks, connect the paired readback/fsync receipt to the output GTC root, and install Integrity/Codec/Output after-values together.

## 2026-11-14 D0 frame-tail correction and full-input capture

The selected D0 tail now reports the actual `nauxbits` span, then `auxdatae=0` and `crcrsv=0`; `encinfo` remains in BSI. Focused D0 assembly tests pass 5/5, selected 5.1 all-rate regression passes 1/1, and a corrected-source Release build completes.

User-directed Release attempt `1794654566716793500` captured 8,417 AUs (56,023,552 bytes) from the Love Song ADM BWF. Wash's OpenJOC syntax-only review found valid I0/D0 frame structure and `malformed_aus=0`, while JOC remains absent: EMDF containers=0, addbsi-signaled AUs=0, and 50,497 skip fields unresolved. Decoder/render/playback remain `NOT_RUN`; candidate state is experimental and `production_allowed=false`. The immediate next increment is the complete JOC EMDF container producer and its D0 block-2 skipfld insertion with the approved 511-byte bound.

## 2026-11-14 D0 frame syntax correction

The existing F13/F14K path already validates the same-source complete EMDF container, enforces C=1..511, passes protected bytes into the D0 PCM-frame producer, and inserts that container once in block 2. OpenJOC 0.20.1's `skip_observed=0` led to the actual syntax fix: the D0 writer had placed the 6 coupling-state bits and 24 D15 `chexpstr` fields in each block rather than in `audfrm`. It now writes them once in `audfrm`; per-block prefixes no longer duplicate those fields. D0 audio-frame (4), frame assembler (5), frame-control (2), and selected 5.1 all-rate (1) tests pass; fmt check passes. Rebuild Release and run a unique attempt before assessing OpenJOC again. The prior candidate remains an experimental pre-fix artifact.

## 2026-11-14 Flat7x aggregate F15 output configuration

- Added profile-aware F15 admission for the CLI-resolved Flat7x layout. The output projection uses the pair target's aggregate 1,664-kbps / 6,656-byte AU extent; the carrier profile remains the D0 1,024-kbps row. OutputConfiguration accepts this exact aggregate extent, and ProgrammeOwnerRoots READY validates it. The selected 5.1 provider path is unchanged.
- This prepares the correct output-owner configuration for the in-memory pair proposal. The CLI still stops before the persistent paired output ledger, durable append receipt and parent output-root swap. No file was created.
- Workspace check, CLI test-target compile, generated-map check and boundary check passed. Clippy reports 18 existing errors and 1 warning outside the changed files; no test binary ran.
- Next: persist the pair F15 ledger across AU callbacks, connect its pair-specific readback/fsync receipt to the output GTC root, and install paired Integrity/Codec/Output after-values after each durable append.

## 2026-11-14 D0 audfrm syntax correction and Release attempt

OpenJOC 0.20.1's `skip_observed=0` led to the ETSI Annex E.1.2.3 placement fix: six coupling-state bits and 24 D15 `chexpstr` U2 values now reside in `audfrm` after `skipflde` and before `audblk`; per-block prefixes no longer duplicate these 54 bits. The selected 5.1 all-rate route remains unchanged. Focused checks passed: D0 audio-frame 4/4, frame assembler 5/5, D0 frame controls 2/2, selected 5.1 all-rate 1/1, and fmt check. Commit `6f3cb91e39d9a8fd308b2ed5f0188d7f67babd10` is pushed.

Corrected Release attempt `1794658627622854200` captured 8,417 AUs / 56,023,552 bytes, 6,656 bytes per AU, and CRC residual 0. The first published source carried a 288-byte protected F13 container. OpenJOC 0.20.1 review is pending; candidate remains experimental with `production_allowed=false`, and render/playback remain `NOT_RUN`.

OpenJOC still reported `skip_observed=0` for that candidate. ETSI TS 102 366 V1.4.1 Annex E.1.2.4 orders `dithflag` before `dynrnge`; the D0 audblk writer now follows this order. The targeted D0 audio-frame (4), frame-assembly (5), frame-control (2), selected 5.1 all-rate (1), `cargo check -p jocforge --locked`, and fmt checks pass. The CLI carries the typed AU0 D0 frame receipt to the progress log. Short diagnostic attempt `1794661230282761900` captured 137 AUs / 911,872 bytes. AU0 receipt is audfrm `[94,171)`, skipflde bit 104, block-2 skiple bit 12,743, U9 skipl at 12,744 with C=230, EMDF `[12,753,14,593)`, nauxbits `[32,494,32,750)`, auxdatae 32,750, crcrsv 32,751, CRC2 `[32,752,32,768)`. OpenJOC 0.20.1 still reports `skip_observed=0`, 822 unresolved fields, zero EMDF/JOC, `malformed_aus=0`; first parser boundary divergence remains open. The short input is diagnostic; Love Song remains the target.

## 2026-11-17 ID14 static row-normalizer slice

Implemented `joc::id14_row_normalizer` from Wash supplement `id14_row_normalizer_static_formula_v1` (manifest SHA-256 `4a78273f32c16e1c93b18529a7f17ebc707391b2fdba93acf2bc41ed7246a4a2`). It validates 1..128 six-byte caller rows; classifies the signed byte at +2; ignores +3; groups on little-endian U16 at +4; emits type-2 marker words and prior-count padding. The compiled target is `x86_64-pc-windows-msvc` / little-endian, so the U16 read is explicit little-endian.

The 29 sealed input-only formula vectors and a local extent-rejection test pass (2/2). Domain check, fmt check, and selected 5.1 all-rate regression pass. Domain Clippy with `-D warnings` reports 13 existing errors in unchanged modules; the new normalizer does not appear. The package semantic row identity remains UNBOUND; this module does not map ADM/D0 roles, provide seven coefficient rows, call the ID14 writer, or emit a bitstream. No encoder was started.

## 2026-11-15 D0 block-two source-generation receipt

- The single E-1037 Release attempt on `D:\Love Song 方大同.wav` completed Meter prepass (8,420 callbacks; selected LUFS -16.13749), then stopped at `Flat7xCoefficientSourceUnavailable` after g=0/g=1 priming. Exit code 2; elapsed 136,733 ms. Its attempt directory is empty: no candidate, partial file, or OpenJOC inspection.
- The D0 block-two preassembly input now requires a Codec-owner-issued `Flat7xD0SourceGenerationReceipt`, an F13 lineage snapshot carried through F14K, F14K source/dispatch/AU identity, and the exact protected container. F13/F14K receipt mismatch and source-generation mismatch reject before D0 frame bytes or cursor are staged. Metadata owner before/after generations, consumer/producer transactions, ID14 producer/writer generations, Codec owner generation, D0 source generation, and the later F15 pair label remain separate fields and types.
- External `Flat7xCoefficientRowsProviderOutput` now requires a provider-owned source lineage/transaction and Codec-owner-issued D0 source-generation receipt; the application checks it against the current expected receipt. OpenJOC index1 defines target order `[L,R,C,Ls,Rs,Lrs,Rrs]`; the current five-carrier-to-seven-row source map remains UNBOUND, so production still returns `Flat7xCoefficientSourceUnavailable`.
- Focused verification passed: D0 receipt/frame assembler 8/8, D0 audio-frame 4/4, provider row/source receipt tests 3/3, application provider bridge 1/1, F13/F14K/F15 pair test 1/1, and selected 5.1 all-rate route 1/1. `cargo fmt --all -- --check` and workspace `cargo check --locked` pass. Workspace Clippy `-D warnings` still reports 28 findings in unchanged files; boundary check still reports the unchanged direct-slice inventory for `crates/jocforge-domain/src/meter/standalone_root.rs`.
- No encoder or OpenJOC process was started. Production 15x7 coefficient mapping and values remain UNBOUND; the selected 5.1 path remains available.
- `SevenCarrierPcmInput` now exposes borrowed ADM rows in target order `[L,R,C,Ls,Rs,Lrs,Rrs]` from an independent JOC-role selection on the source role map. The graph fixture resolves to `[0,1,2,4,5,6,7]`; I0 remains `[L,C,R,Ls,Rs,LFE]`, D0 remains `[Ls,Rs,Lrs,Rrs]`, and physical N-row input retains the other rows. This is a shape-only value; QMF/window history and 15x7 coefficient production remain unavailable, so the CLI still fails closed.
- The one-hot row-selection test passed 1/1, workspace check and fmt check pass. Clippy reports existing domain findings; boundary check reports the existing `meter/standalone_root.rs` inventory discrepancy.
- `SevenCarrierWindowSet` now analyzes each selected ADM PCM role independently through the shared QMF row analyzer, retaining its own nine-row analysis history and previous 24-slot spectrum plane. It emits source-bound 48-slot raw-QMF windows in target order. The one-hot lane-isolation/history-rollover tests passed 2/2; existing mode-0 fitting tests passed 2/2 and the selected 5.1 all-rate route passed 1/1. Workspace check/fmt/diff pass. C=7 fit and production integration remain unavailable.
- Normal-equation construction, LDL solve/normalization, score evaluation, and support-growth search now accept a fixed carrier count. The existing C=5 functions remain wrappers; C=7 uses seven-word equation/solution vectors and target-role-order support growth as a diagnostic policy. C=7 equation/solver shape tests and the 5.1 all-rate route pass. Existing solver/score fixture tests still show `ProviderIdentity`/manifest mismatches in unchanged fixtures.
- C=7 fit rows now pass through representation retry and selector-1 ID14 writing. The `SelectedJocState` after-value retains seven independent QMF histories in a length-prefixed `J7QMF001` canonical extension; C5 states keep the original JOCSEL01 bytes. The real CLI source callback now selects C7 fit only for the Flat7x profile and binds it to the Codec D0 source-generation receipt. Synthetic selector-1 writer and JOC owner-extension tests pass; no real ADM candidate or OpenJOC inspection has run yet.
- E-1092 short ADM produced a 137-AU candidate and OpenJOC 0.20.1 rendered an 8-channel 7.1 WAV; OpenJOC admitted 137/137 under observed-vendor compatibility. Wash reports that `DynamicOnly + LFE` expresses one `Speaker(RcLfe)` plus 15 dynamic anchors; the `Mixed bed + 15 dynamic` scratch guard appears overstrict, while ID14 row-to-OAMD identity remains open. The exact-two-item payload predicate is also too strict: additional IDs are valid when exactly one ID11 precedes exactly one ID14. E-1093 full Love Song stopped before AU append at F14/F14K with `EMDF_PAYLOAD_CAPACITY_EXCEEDED`; C7 retry now uses the 372-byte inner ID14 parser limit while the complete skipfld remains limited to 511 bytes. E-1095 passed Meter and appended 19 AUs (126,464 bytes); the local parser incorrectly applied the five-channel sparse channel limit to selector 1. Position7, modulo 7 and a channel-count-based initial U3 guard now cover the seven-carrier branch; E-1097 reproduced the remaining fixed `>4` guard before it was updated. E-1096 used a temporary Full-only retry policy, passed 513 AUs, then C7 search failed on the first scaled-I32-overflowing support candidate. C7 now skips unrepresentable candidates without clamping and keeps searching; C5 behavior remains unchanged. A fresh full-song attempt is required.
- E-1098 completed the full Love Song encode: 8,417 AUs / 56,023,552 bytes / 269.344 s. OpenJOC 0.20.1 parsed/admitted 8,417/8,417 under observed-vendor compatibility, with 0 malformed AUs and continuous timing. Its 7.1 render produced 12,928,544 frames at 48 kHz, 8-channel IEEE float32, LFE index3. The selected 5.1 all-rate profile/frame-size regression passed 1/1. Renderer maturity is experimental and semantic binding unresolved; the attempt is not product-exposed. ETSI strict still flags ID11 wrapper fields and `codecdatae=0` on ID11/14. Remaining: user listening, ID14-row/OAMD-slot identity, retained block2 placement receipt, and ID11 wrapper alignment.
- E-1099 was externally stopped after 3,217 AUs / 21,412,352 bytes; OpenJOC admitted its partial 3,217/3,217 with zero malformed AUs, and strict reported only `codecdatae=0`. E-1100 completed 8,417 AUs and rendered a 12,928,544-frame, 8-channel 48 kHz float WAV. Flat7x ID11 wrapper flags now match the official scalar comparison; selected 5.1 keeps its prior wrapper profile. OpenJOC reports 16 ordered metadata slots: index0 non-dynamic, indices1..15 dynamic; explicit Speaker(RcLfe) identity remains unexposed. The AU0 D0 placement receipt carries block2 bits 12201..15979, PublishedAuIndex0, and Codec owner source-generation receipt3; separate F15 source-generation label2 is source-index-derived. These typed domains are intentionally not equated. The receipt is not yet checked by the Codec after-root proposal. ETSI strict now flags only `codecdatae=0` on ID11/14; observed-vendor compatibility passes. Remaining: user listening, row-to-speaker identity, Codec receipt-to-after-root join, renderer semantic binding.

## 2026-11-16 I0 source, BAP, and bitrate-fit status

E-1109 completed a full Love Song Flat7x encode after routing provisional I0 ADM rows into F07-F10, but OpenJOC internal-base `L/R/C/LFE` RMS remained zero. E-1111/E-1112 scalar diagnostics confirmed the mapped ADM rows and F08/numeric coefficients carry signal at an active source; the original I0 block policy forced blocks 1–5 to zero BAP. A short q=400 probe (E-1119) produced nonzero OpenJOC core `L/R/C` at a 30-second Love Song window. E-1120 performs bitrate-fit selection over same-source selected-core q trials, bounded by the accepted selected 5.1 q and the I0 20,480-bit / 2,560-byte frame. The 8-AU search selected q=400 and matched E-1119's candidate hash; OpenJOC admitted 8/8 with no malformed AUs and decoded nonzero L/R/C. That segment's LFE source was quiet.

E-1121 completed the full Love Song run with 8,417 AUs / 56,023,552 bytes; CRC residuals were zero and all 8,417 D0 block-two placement receipts were within C≤461. OpenJOC observed-vendor compatibility passed; decoder admission is 8,416/8,417, with AU 6721 reported as `AuxDataNotByteAligned(1004)`. E-1105/E-1109/E-1121 D0 frame bytes are identical throughout; AU 6721 differs in the I0 AC-3 frame, which OpenJOC reports as legacy-independent. Wash confirms ETSI V1.4.1 §4.4.4.1 permits non-byte-aligned AC-3 auxbits and identifies that OpenJOC 0.20.1 applies its E-AC-3 EMDF auxiliary classifier to every AU frame without a LegacyIndependent gate. The exact I0 raw-span semantic join remains UNBOUND, so the parser error does not alone establish a syntax violation. Full OpenJOC decode/render and listening remain NOT_RUN after AU 6721. Selected 5.1 stays unchanged; targeted q-policy and 5.1 all-rate tests pass. Role mapping is provisional; product exposure stays disabled.

A diagnostic parser probe was built from the exact OpenJOC v0.20.1 tag source `eb5cdd5562017b7ec4d54bb243dd3a626310ecb9`, with only the LegacyIndependent frame-end EMDF classifier gated (patch SHA-256 `45e222043b9fc470b0251b9a21f9c4273f00aa909ac54dfea9799b6a8b65298b`, probe binary SHA-256 `d7e8afc8b28349e23afbeaace51b8c1985ad0f88a832929d160345b4e729b355`). The unchanged E1121 candidate parsed/admitted 8,417/8,417 and rendered a 7.1 diagnostic WAV. Internal-base core RMS is `[L=.03049982,R=.03160754,C=.08086876,LFE=.00557430]`; relative to E-1106 internal base, six windows show a recurring approximately -5,120-sample lag. Core audio is now present; the exact typed source-time alignment remains the next issue. The parser-probe render is diagnostic only; official binary admission and subjective listening remain unresolved.

Callback/source timestamps are distinct types in the current code: input rows at callback `g` carry `first_sample=g×1536`, `source_for_callback(g)=s=g-2`, and `publication_for_source(s)=AU j=s-2`. Flat7x I0 currently feeds those callback rows directly into F07. The parser-probe internal-base comparison against raw PCM rows yielded best lags -4,864 samples for I0 L/R/C and -5,888 for D0 Ls/Rs/Lrs/Rrs at multiple windows; E-1106 core peaked at +256. The typed source-window-to-AU sample join remains `UNBOUND`; these measurements identify the next Wash clock review, not a license to hardcode a shift.

## 2026-11-16 Flat7x I0 source-extent receipt

The Flat7x-only `Flat7xI0AnalysisInput` now carries the validated `Flat7xD0PcmSourceExtent` from the same reader callback, including source/callback indices, `first_sample`, valid/padded counts, and origin. The CLI requires exact equality with the extent on `Flat7xSourcePcmProjection`. `Flat7xPairI0FrameCandidate` and the paired AU candidate retain that extent; I0 frame staging checks its callback/source against the pair binding, AU composition checks I0 against the binding, and the F12 pair stage checks I0 against the D0 frame extent. The shared selected 5.1 analysis/F04 route is unchanged.

This receipt binds the reader window consumed by Flat7x I0. The sample-window-to-published-AU conversion remains `UNBOUND`; no PCM shift or q/audio policy change was made. `cargo fmt --all -- --check`, `cargo check --workspace --locked`, the new I0/projection extent test (1/1), and selected 5.1 multi-rate regression (1/1) pass. A feature-gated pair integration test stops earlier at its existing synthetic q=400 budget input (22,518 required bits vs 20,480 available), before reaching the new extent assertion. No encoder run was started.


## 2026-11-16 E-1123 candidate result

The one-shot E-1123 Release run on commit add93721527c3aa915e8729de0ecb5d69cac6476 produced 8,417 AUs / 56,023,552 bytes. Candidate SHA-256 81071d12024295e31e2215c2d4edd512e5110fb7fdffe0acb491a42d6a72566f is byte-identical to E-1121. D0 block-two receipts: 8,417/8,417; max C=461; overflow=0; AU CRC residual=0. Attempt status remains experimental: 7 pending drop receipts, programme_done=false, production_allowed=false, render/playback NOT_RUN. The E-1121 OpenJOC 0.20.1 parser result applies to the identical bytes; E-1123 itself was not reinspected. No new render or listening was performed.


User listening feedback for E-1123: it sounded as bad as the previous version. The candidate SHA-256 matches E-1121 exactly, so encoded audio bytes did not change; no objective similarity score or new render is recorded.

## 2026-11-17 Wash timebase boundary

Wash records DEE file_position start/end as seconds minus offset, with validation using round(sample_rate x seconds) and stored signed-64 sample positions using truncation. embedded_timecode omits offset subtraction; FFOA is independent. The first source-range consumer selects source-array positions, but owner/generation, endpoint convention, connection to AtmosReader cursor/limit, and join to JOCForge first_sample/source/AU remain UNBOUND. Do not use correlation peaks as a timing correction.

## 2026-11-17 comparison and D0 boundaries

The user reports E-1123 sounds as bad as the previous version; its bytes are identical to E-1121. The keyless candidate is hash/size-matched to GENERATION.json, but the generation record is not bound to the Python run or current entrypoint/source. The reported Rust failure artifact remains USER_REPORTED_FAILURE_FILE_ID_UNBOUND.

Wash source evidence binds DEE file_position adjustment and sample conversion but not the JOCForge sample-window-to-AU join. Do not infer a row shift from the correlation lag alone.

D0 record-vector begin/end pointer operations and the first opaque skipfld record owner are now registered in the shared handoff; U9 syntax, block identity, complete C, and native overflow behavior remain UNBOUND.

## Flat 7.1 in the browser bridge (measured 2026-11-23)

- `encode_adm_resource(resource, rate_kbps, object_count, job_xml, layout)` carries the layout; FFI setters `set_rate`, `set_objects`, `set_job_xml`, `set_layout`.
- 5.1 default is unchanged by every step: `layout=0 -> 32768 B, sha256 30581464dc3e2203…`.
- flat7x gates found in order (each one measured, not inferred):
  1. `validate_flat7x_experimental_candidate_path` — skipped on wasm (memory has no candidate file name).
  2. `MemoryOutputStaging::append_flat7x_au` was still the retirement stub; tag `first=abandon-site-2` exposed it. The real memory implementation now runs (offset write, extent check, readback, pair receipt) via `append_flat7x_au_to_memory` (promoted from the test-only helper).
  3. Current failure: `first=flat7x-memory-append` — the shared memory append helper rejects the first pair. Next: compare `write_and_readback_sink` preconditions (offset seeding, sink length, digest window) against the file adapter, which writes into a pre-sized file.
- Bridge detail strings are bounded to 256 chars, so diagnostics must be prefixed, not appended.

### Flat 7.1 bridge, round 13 (measured 2026-11-23)

- `Flat7xAuAppendIoFailure::kind()` added (public classifier) because `error` is private.
- First failure was `receipt-rejected`: the memory pair helper reported `file_fsync = false`, so the
  domain rejected the pair receipt candidate. Passing `Ok(true)` is correct for an owned buffer —
  the bytes and the readback are the same storage, so evidence is complete once the comparison passes.
- After that fix the pair chain runs and flat7x advances to
  `Flat7x experimental candidate capture` — the remaining fs-bound step (candidate `.ec3` write and
  `std::fs::read(staging.staging_path())`).

### Flat 7.1 works in the browser bridge (measured 2026-11-23, round 15)

Chain of measured gates, each fixed in order:

1. `validate_flat7x_experimental_candidate_path` — skipped on wasm (no candidate file name in memory).
2. `MemoryOutputStaging::append_flat7x_au` — was still the retirement stub; replaced with the real
   memory append (`append_flat7x_au_to_memory`, promoted from the test helper).
3. `receipt-rejected` — the memory helper reported `file_fsync = false`; an owned buffer's readback is
   its own durable evidence, so it now reports complete.
4. Candidate capture — `std::fs::read(staging.staging_path())` replaced by `staged_stream_bytes()`
   (wasm-only borrowing accessor, copies out of the memory core) and the `.candidate.ec3` write path
   replaced by `stage_flat7x_elementary_stream_to_memory` (new, io crate).
5. Run product — the flat7x branch returned `RunProduct::Published`; on wasm it now returns the staged
   bytes so `encode_adm_resource` can hand them to the bridge.

Measured on `.test-tmp/wasm-parity/excerpt.wav`:

- `layout=0` (5.1, 1024 kbps, 15 objects): 32768 B, sha256 30581464dc3e2203… — unchanged since the
  start of this work.
- `layout=1` (flat 7.1, 1024 kbps): 53248 B, sha256 4ad19e7d8b28fc21…

NOT yet verified: decoding the flat7x stream with OpenJOC, the six carrier-rate variants, and the
byte-for-byte comparison against the native CLI (`--joc-layout flat7x --total-rate …`).

### Flat 7.1 parity check, round 16 (measured 2026-11-23)

Native reference (`jocforge.exe encode <abs in> <abs out>.candidate.ec3 --joc-layout flat7x --total-rate 1024`):

- 32768 B, sha256 `ba07f48efad20ed6…`, `emdf_payload_sequence=flat7x_experimental_joc_pair_v1`,
  `d0_block_two_placement_receipts=8/8`, `production_allowed=false`.

Wasm bridge (`set_layout(1)`, default rate 1024 codec rate):

- 53248 B, sha256 `4ad19e7d8b28fc21…`.

They differ, so flat 7.1 is not yet at parity. Two candidate causes, both unverified:

1. Rate semantics: the CLI takes `--total-rate` (1024…1664, I0 640 + D0 1024 by default) while the
   bridge maps `rate_kbps` onto `CodecRateProfile` (384…1024). The two axes are not the same.
2. AU count: 32768 B looks like 8 aggregated frames, 53248 B like 13 — the wasm run may be publishing
   a different number of pairs, which would also change the trailer.

Next action: give the bridge a `total_rate_kbps` parameter that maps onto the Flat7x carrier plan, then
re-run the comparison before touching the site UI.

### Flat 7.1 parity reached (measured 2026-11-23, round 17)

The mismatch in round 16 was an axis error, not a bridge defect: `--total-rate N` maps to codec rate
`N - 640` (`CodecRateProfile::from_kbps(total - 640)` in `crates/jocforge-cli/src/main.rs:321`).
Re-running the bridge with codec rate 384 (= total 1024) matches the native CLI exactly:

| layout=1 codec rate | total rate | wasm | native CLI |
| --- | --- | --- | --- |
| 384 | 1024 | 32768 B, `ba07f48efad20ed6…` | 32768 B, `ba07f48efad20ed6…` — **identical** |
| 448 | 1088 | 34816 B, `a6bf2da2e4f342cb…` | not run |
| 576 | 1216 | 38912 B, `a770867b037e3ad0…` | not run |
| 640 | 1280 | 40960 B, `96a3985d598cbe1a…` | not run |
| 768 | 1408 | 45056 B, `f8d2a64e4abbbc8d…` | not run |
| 1024 | 1664 | 53248 B, `4ad19e7d8b28fc21…` | not run |

5.1 default still `30581464dc3e2203…` in the same build.

UI implication: the site must present **total rate** (1024–1664) and send `total - 640` as `rate_kbps`.

Still open: fmt/clippy/test/`check-boundaries.py` on the touched crates; OpenJOC decoding of the
flat7x stream; ADM OAMD compatibility of the browser-produced candidate.

### Suite status after the Flat7x bridge work (2026-11-23, round 18)

- `cargo fmt --all --check`: clean.
- `cargo check --workspace --all-targets --locked`: errors 0; warnings only in pre-existing test
  targets (`jocforge-application` test helpers, `jocforge-domain` `flat7x_d0_*` tests).
- `cargo test --workspace --locked`: all green (largest suites: 688 in domain, 65, 18, 14, 9, 6, 5…).
  One compile break had to be fixed first: the pipeline test helper called
  `jocforge_io::flat7x_au_append::append_flat7x_au_to_memory_for_test`, renamed to
  `append_flat7x_au_to_memory` when the helper was promoted to production.
- `scripts/check-boundaries.py`: **fails** with `boundary check failed: substring not found`. The
  audit asserts exact literals, so the edited Flat7x paths (product branch in `run_inner`, candidate
  capture, memory staging) no longer match its expectations. This must be resolved before the work
  can be called complete; the script swallows the traceback, so the next action is to run the checks
  with the assertion made verbose (or to grep the audit for the Flat7x literals it pins).

### Boundary audit green again (measured 2026-11-23, round 19)

`check-boundaries.py` now reports: `boundary checks passed (91 locally verified scoped tasks;
104 inventoried IDs; 0 unmaterialized roots; 6 partial roots)`.

Two things had to change, and both are honest consequences of this work rather than workarounds:

1. The audit's F12 check pinned `fn append_flat7x_au_to_memory_for_test<`; the helper was promoted to
   production as `append_flat7x_au_to_memory<`, so the expectation was updated.
2. The same check pinned `|_| Ok(false)` for the memory sink's durability evidence. An owned buffer's
   readback *is* its durable evidence, so the helper now reports `Ok(true)`; the expectation and its
   description were updated to say that.

The audit also had to be made honest about failures: it caught `(BoundaryError, Exception)` and printed
only `str(e)`, which reported "substring not found" with no location. It now prints the traceback for
unexpected exceptions and keeps the one-line message for real boundary violations.

### Topology guard located (round 22)

ncode_request.rs:1515-1527 is the only wasm guard rejecting 	opology. Native equivalent uses a tempdir carrier plus NamedTempFile; on wasm ComposedOutput is already Cursor<Vec<u8>> and the carrier path can be memory://audio.candidate.ec3. Flipping it needs the admission product-name check and the composition finalize path confirmed first.


### Topology blockers verified (round 23)

- Admission: `memory://` is namespace-only, so `memory://audio.candidate.ec3` passes the product-name rule.
- Composition finalize (`encode_request.rs:2042`) is entirely `#[cfg(not(target_arch = "wasm32"))]`:
  length check via file metadata, CMAF packaging via `NamedTempFile`, then rename. On wasm the
  composition is never consumed, so removing the topology guard would report success while silently
  dropping the composed programme.
- A wasm composition finalize must exist before topology is exposed; there is no `set_topology` FFI yet.

### Composition finalize: exact insertion map (round 25)

- wasm guard to relax: ncode_request.rs:1515-1527.
- composition bytes are produced by the existing producer call at ncode_request.rs:1772 (composition.as_mut()), so a Cursor<Vec<u8>> accumulates the composed programme.
- wasm run product today: ncode_request.rs:2207 let product = RunProduct::Bytes(staging.into_product_bytes());. With topology present this silently returns the staging bytes instead of the composed ones, which is the drop identified in round 23. The wasm product must prefer the composition sink: Some((_, sink)) => RunProduct::Bytes(sink.into_inner()).
- blocking gap: there is no set_topology FFI and no 	opology parameter in ncode_adm_resource, so the branch cannot be exercised from JS. Both must be added in the same change as the two edits above, otherwise the work is untestable.
- CMAF (cmaf_rate.is_some()) with topology stays rejected on wasm: packaging is NamedTempFile-based.


### Topology path runs in wasm (round 28)

- set_dependent_count(1) on the bridge produces a distinct composed stream: 32768 B, sha256 22daab711d98072b....
- Without it the baseline is untouched: 32768 B, 30581464dc3e2203....
- Native reference with --dependent-count 1: 32768 B, 30a99c1b9604565... — same size, different bytes, so parity is NOT reached.
- Most likely delta: the CLI chains with_emdf_location(...) and a block-partition step onto SubstreamPlan::new (main.rs:255-270); the bridge currently sets only (layout, dependent_count, LfeOwner::Independent).
- Next: replicate the CLI plan chain in the bridge (EMDF location + block partition), then re-compare.


### Topology parity confirmed (round 33)

The mismatch was the EMDF carriage profile, not the plan chain:

- wasm bridge (hardcoded EmdfCarriageProfile::DeployedCompat): 32768 B, 22daab711d98072b...
- CLI --dependent-count 1 --emdf-profile deployed: identical, 22daab711d98072b...
- CLI default profile (etsi/strict): 30a99c1b9604565...

So set_dependent_count(1) is byte-identical to the CLI when the profile is stated. Comparisons must pass --emdf-profile deployed to match the bridge.

Builder signatures (domain substream_topology.rs): with_emdf_location(self, ..) -> Result<Self, String> (177), with_partition(self, ..) -> Self (185), which is why the chain is nd_then then map.

Still open: fmt/check/test/check-boundaries after this batch; JS/UI exposure of the dependent count; speaker export.


### Uncommitted work inventory (end of session)

`
M crates/jocforge-io/src/atomic_output/memory.rs
 M crates/jocforge-io/src/flat7x_au_append.rs
 M crates/jocforge-io/src/flat7x_elementary_stream_stage.rs
 M crates/jocforge-pipeline/src/encode_request.rs
 M crates/jocforge-pipeline/src/output_publication.rs
 M crates/jocforge-pipeline/src/source_execution.rs
 M crates/jocforge-wasm/src/ffi.rs
 M crates/jocforge-wasm/src/lib.rs
 M docs/memory/agents/implementation-agent.md
 M scripts/check-boundaries.py
?? crates/jocforge-pipeline/src/encode_request.rs.bak
?? crates/jocforge-wasm/src/ffi.rs.bak
?? crates/jocforge-wasm/src/lib.rs.bak
`

Every verified capability in this session lives in these uncommitted files. AtmosLabs (the site) has no repository at all yet.

### Committed and extended (2026-11-23, rounds 34-35)

The inventory above is historical: the bridge work landed as `e079e0ecb`, and AtmosLabs now has its
own repository (three commits at the time of writing). Two capabilities were added on top.

**Speaker export (OpenJOC side, not this repo).** `openjoc-wasm` hardcoded the speaker layout to
`2.0` for every non-binaural renderer, so `RenderMode::Speaker` silently downmixed. OpenJOC commit
`1313141` threads the layout through and adds
`openjoc_wasm_decoder_create_with_speaker_layout(dialnorm, layout_code)`. Measured there: 5.1 → 6
channels, 7.1.4 → 12, bit-identical to a native `OpenJocSession` in speaker mode.

**Surround EX over the browser bridge.** The guard in `run_worker_with_object_count` only checks
`joc_layout != Flat7x || topology.is_some()` — nothing about the target — so flat 7.1 without a
dependent count was already admissible; the bridge just hardcoded `surround_ex = 1`. Changes:

- `encode_adm_resource` takes `surround_ex: u8` (was the literal `1`).
- `Encoder::set_surround_ex` + FFI `jocforge_wasm_encoder_set_surround_ex`.

Measured on `.test-tmp/wasm-parity/excerpt.wav`, flat 7.1, codec rate 384 (= total 1024),
`--emdf-profile deployed`:

| `surround_ex` | wasm bridge | native CLI |
| --- | --- | --- |
| 1 not encoded | 32768 B, `ba07f48efad20ed6…` | identical |
| 0 unspecified | 32768 B, `e1df9b4f8ef9756c…` | identical |
| 2 encoded | 32768 B, `bb17e600cbe4e611…` | identical |

The default (1) still equals the pre-existing flat7x anchor, so nothing on the default path moved.
Modes 1 and 2 differ by 80 bits confined to byte offsets `[2, 3, 11]` of every 4096-byte block — the
I0 BSI suffix, no audio data. 5.1 with `surround_ex = 2` is rejected with
`Surround EX requires native flat7x output`.

Two tests still asserted the superseded memory-sink contract and were failing before this batch:

- `jocforge-io` `in_memory_sink_rejects_non_durable_fully_receipted_pair_commit` →
  `in_memory_sink_commits_a_receipted_pair_with_complete_evidence` (asserts `file_fsync = true`,
  non-zero readback digest, buffer equals the whole pair).
- `jocforge-pipeline`
  `flat7x_staging_routes_receipted_pair_to_memory_sink_and_rejects_nondurable_commit` →
  `…_without_touching_the_filesystem` (the memory route must still never create the staging file).

`check-boundaries.py` pinned the old test name and the old
`AppendReceiptCandidateMismatch` reason; both pins were updated to the new contract

Gates for this batch: `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets
--locked` exit 0 with no warnings inside `jocforge-wasm/src`, `cargo test --workspace --locked` exit
0 with no failures, `check-boundaries.py` `91 / 104 / 0 / 6`.

### Streaming input, increment 1: byte-identity harness and the data-window seam (2026-11-23)

Goal: the browser must stop holding the whole ADM in wasm memory, **without changing a single
output byte**. Increment 1 lands the guarantee and the seam; it changes no encoded bytes.

**Golden vectors.** `scripts/golden-vectors.py` runs the release CLI over 16 flag-only cases
(six 5.1 rates, six flat-7.1 total rates, the three Surround EX declarations, and
`--dependent-count 1`) and records `{bytes, sha256}` in
`docs/memory/agents/golden-encode.json`. `--update` rewrites, default verifies and exits non-zero on
any mismatch. Job-XML cases are deliberately excluded: their XML carries absolute paths, so they are
not stable across directories. Every case reads the whole PCM, so any block-offset or length
deviation would change the product sha.

Recorded baseline (all reproduce the anchors already in this file):

| case | bytes | sha256 |
| --- | --- | --- |
| 5.1 384 / 448 / 576 / 640 / 768 / 1024 | 12288…32768 | `17411893` / `2bb2bcfc` / `ecf7cd3e` / `89f4a4bc` / `09888000` / `30581464` |
| flat7x total 1024…1664 | 32768…53248 | `ba07f48e` / `a6bf2da2` / `a770867b` / `96a3985d` / `f8d2a64e` / `4ad19e7d` |
| flat7x ex 0 / 2 / 1 | 32768 | `e1df9b4f` / `bb17e600` / `ba07f48e` |
| topology dependent 1 | 32768 | `22daab711d98072b` |

**The seam.** `ParsedRiffPcm24` no longer holds `data_payload: &'a [u8]`; it holds
`RiffDataWindow<'a>`:

- `Resident(&'a [u8])` — the original slice. Reads stay zero-copy (`Cow::Borrowed`), so every
  current caller behaves exactly as before.
- `Streamed { reader: &'a Mutex<Box<dyn RiffDataReader>>, len }` — reads come from a reader and are
  copied (`Cow::Owned`). `RiffDataReader::read_exact_at(offset, len)` measures `offset` from the
  first data-chunk payload byte and **must** return exactly `len` bytes; a short read is rejected
  rather than silently zero-filled.

`read_pcm24_chunk(&self, …)` keeps its signature, so `frontend_pcm_source`, `meter_source_cursor`,
`meter_prepass` and the rest compile untouched. `streamed_window(&parsed, &reader)` re-serves an
already parsed extent from a reader; `FileRiffDataReader` (native only) is the seek+read_exact
implementation used to prove equivalence.

Two decisions worth keeping: the window borrows the reader instead of owning it (owning gave
`ParsedRiffPcm24` drop glue, which broke NLL at `encode_request.rs:1703` — `prepared` "does not live
long enough"), and `RiffDataReader: Send` with `Mutex` instead of `RefCell` (the pipeline requires
`Sync`).

Measured: `streamed_window_decodes_exactly_the_resident_blocks` decodes every block of a synthetic
4096-frame RIFF twice — once from the resident slice, once through a file reader — and asserts the
two planar row vectors are identical. `streamed_window_rejects_a_short_reader` proves a reader that
returns fewer bytes fails with `InvalidInput`. All 16 goldens unchanged after the seam.

**Not done yet, in order:**

1. **Parse from a reader.** `parse_selected_riff_pcm24(bytes: &[u8], …)` still needs the whole file
   to walk chunks and hash the source (`source_sha256 = Sha256::digest(bytes)`). Streaming needs a
   chunk walk that reads 8-byte headers and skips the data payload, plus a streaming hash in file
   order. Metadata payloads (`axml`/`chna`) are currently borrowed slices; they would become owned.
2. **Wasm pump.** A resume-free synchronous pipeline cannot call JS mid-call, so the reader needs
   another thread to service it: `SharedArrayBuffer` + `Atomics.wait` in the encoding Worker, with a
   second Worker (or the main thread) filling the buffer. That requires cross-origin isolation
   (`Cross-Origin-Opener-Policy: same-origin` + `Cross-Origin-Embedder-Policy: require-corp`) on the
   nginx site.
3. **JS driver.** `Encoder` currently accumulates every pushed byte. It must instead accept a
   `File` handle, stream header + tail + sequential data ranges from `file.slice(...)`, and report
   progress per range.

Re-run `python scripts/golden-vectors.py` after each of those; anything that moves a sha is a bug in
the change, not a new baseline.

### Streaming input, increment 2: reader-driven parse and byte-identity binding (2026-11-24)

Increment 2 removes the second resident requirement: the parser no longer needs the whole resource.
All 16 goldens still match byte for byte.

**One reader, file-absolute.** `RiffDataReader` now exposes `read_at(offset, len)` measured from the
first byte of the **file** plus `len()`, so a single reader serves the chunk walk, the source hash and
the PCM block reads. `RiffDataWindow::Streamed` carries the window's `base` and translates a
window-relative read into a file read. Implementations: `SliceRiffDataReader` (resident bytes, also
used by the resident entry point so there is one parse path) and `FileRiffDataReader` (seek +
read_exact, native only).

**Reader-driven walk.** `walk_riff(read, window_builder, requested_metadata_id)` walks 8-byte chunk
headers through the reader, reads `fmt ` and the metadata payloads, and **skips the `data` payload**
entirely; the builder then installs either a resident slice or a streamed window. Both public entry
points are thin wrappers: `parse_selected_riff_pcm24(bytes)` and
`parse_selected_riff_pcm24_streamed(&Mutex<Box<dyn RiffDataReader>>)`. The hash is still over the
whole resource (`RiffDataReader::sha256`, 1 MiB chunks; the slice implementation overrides it with a
single `Sha256::digest` so a resident parse costs no extra copies).

**Metadata became owned, and the binding became byte identity.** A streaming parse cannot borrow
metadata from a buffer it does not hold, so `RiffMetadataPayload` owns its payload and carries
`payload_offset`. That forced one honest semantic change: `bind_resource_payload` used to derive the
chunk offset by pointer subtraction, which required the payload to be a subslice of the resource
allocation. It now takes the recorded offset and verifies `resource[offset..offset+len] == payload`
plus the 8-byte chunk header. The accepted inputs are unchanged except for one deliberate case: a
copy of *identical* bytes now binds, because the property being enforced is "this metadata is the
metadata of these bytes", not "these two objects share one allocation". `meter_source_bridge`'s test
proves both halves (identical copy binds, one flipped byte is rejected).

Touched for the offset plumbing: `SelectedAdmResourcePayloads::new` takes three `(payload, offset)`
pairs; call sites updated in `encode_request.rs`, `meter_source_bridge.rs`, `meter_input_probe.rs`
and four test helpers.

Assertions updated because the old ones pinned pointer identity or the old hash expression:
`selected_adm_payloads_are_unique_borrowed_spans_from_one_riff` (now byte identity at the recorded
offset), the CLI binding test above, and the F06OF audit pin
(`'source_sha256: Sha256::digest(bytes).into()'` → `'let source_sha256 = read.sha256()?;'` plus
`'Ok(Sha256::digest(self.bytes).into())'`).

New tests: `streamed_parse_observes_exactly_what_the_resident_parse_does` (sha, chunk ids, metadata
payloads, selected payload, timeline, block alignment, channels, data offset, ADM-chunk flag and
every decoded block all equal between the two parses) and
`streamed_parse_rejects_a_resource_whose_extent_disagrees`.

Gates: goldens 16/16 identical, `cargo test --workspace --locked` exit 0, `cargo fmt --check` clean,
`cargo clippy --workspace --all-targets --locked` exit 0, `check-boundaries.py` `91 / 104 / 0 / 6`.

**Still not streaming in the browser.** The pipeline and the bridge ABI still hand over one buffer
(`Encoder::input`), and `prepare_selected_adm_bwf_input` still takes `Arc<[u8]>` for the identity and
the binding check. Next increments: (3) let the application layer accept a streamed parse without a
resident `Arc<[u8]>` — the identity hash and the binding both already have reader-friendly forms;
(4) the wasm pump (`SharedArrayBuffer` + `Atomics.wait` in the encoding Worker, second thread fills,
`COOP`/`COEP` headers); (5) the JS driver over `file.slice`.

### Increment 3 scoping (measured 2026-11-24)

The remaining resident requirement is exactly two places, both measured:

1. **Native input is fully resident too.** `encode_request.rs:557`
   `let resource = read_resource(&request.input_path)?;` followed by
   `parse_selected_riff_pcm24(&resource, None)` at `:558`. So the CLI today reads the whole ADM into
   RAM — a 743 MB file costs 743 MB in the desktop tool as well, not just in the browser. Swapping
   these two lines for `FileRiffDataReader::open` + `parse_selected_riff_pcm24_streamed` is the whole
   native win, and it can be verified against the 16 goldens.
2. **`PreparedJobInput` stores the buffer.** `application/src/lib.rs:91` `resource: Arc<[u8]>`, with
   `resource() -> &[u8]` at `:298`. Every production consumer turned out to use
   `domain_identity().resource()` — that is `ResourceIdentity { input_sha256, resource_bytes }`, not
   the bytes — except one length read at `pipeline/src/meter_source_bridge.rs:66`
   (`u64::try_from(prepared.resource().len())`). PCM reads never touch it: they go through
   `ParsedRiffPcm24::read_pcm24_chunk`.

So increment 3 is smaller than it looked: drop `resource: Arc<[u8]>` from `PreparedJobInput`, keep
`resource_bytes` in the identity (the streamed walk already knows the file length), and pass
`&ParsedRiffPcm24` straight through to preparation instead of an `Arc`. The pointer-identity
assertions in tests (`prepared.resource().as_ptr() == resource.as_ptr()`) become byte-identity
assertions, matching the binding change in increment 2.

Then the ordering that keeps the byte promise checkable at every step: native streamed encode against
the goldens → wasm pump → JS driver.

### Increment 3, first slice: `resource_bytes` on the parsed source (2026-11-24)

`ParsedRiffPcm24` now records `resource_bytes` (the whole-resource length the walk observed) and
exposes it. That is the value `prepare_selected_job_input` needs to build `ResourceIdentity` once the
`Arc<[u8]>` parameter goes away, and the streamed walk already knew it. Asserted in
`streamed_parse_observes_exactly_what_the_resident_parse_does` (both parses report the file length).
Goldens unchanged, workspace tests exit 0, audit `91 / 104 / 0 / 6`.

**Exact remaining diff for increment 3** (measured, mechanical):

1. `application/src/lib.rs`: change `prepare_selected_job_input(…, resource: Arc<[u8]>, metadata)` to
   take `resource_identity: ResourceIdentity`; add `ResourceIdentity::from_bytes(&[u8])` for callers
   that still hold the bytes; drop `resource: Arc<[u8]>` from `PreparedJobInput` and delete
   `resource()` (its only production use is a `.len()` at `pipeline/src/meter_source_bridge.rs:66`).
2. `bind_resource_payload` loses the resource-slice re-read; it keeps offset + payload and computes
   the span. `MetadataPayloadBinding` then only reports an empty payload or an offset overflow. The
   provenance guarantee moves to the parse: `selected_adm_payloads()` returns payloads read from the
   same reader that produced `source_sha256`.
3. Call sites: 31 `prepare_selected_job_input(` (22 of them in `application/tests/prepared_job_input.rs`,
   all passing `Arc::clone(&resource)`) and 6 `prepared.resource()` reads — both are single-token
   rewrites.
4. `encode_request.rs:557` becomes `FileRiffDataReader::open(&request.input_path)` +
   `parse_selected_riff_pcm24_streamed`, which is the actual native win (the CLI currently reads the
   whole ADM into RAM).

Nothing above is started yet; the tree is green at `resource_bytes`.

### Increment 3, second slice: the application layer no longer holds the resource (2026-11-24)

`PreparedJobInput` no longer stores `Arc<[u8]>`; it stores `ResourceIdentity` only, and `resource()`
is gone. The single production use of the buffer was a `.len()` at
`pipeline/src/meter_source_bridge.rs:66`, now `resource_identity().resource_bytes()`.

Signature change: `prepare_selected_job_input(…, resource_identity: ResourceIdentity, metadata)` and
`prepare_selected_adm_bwf_input(programme_id, job_xml, parsed)` — the latter derives the identity from
the parse (`ResourceIdentity::new(parsed.source_sha256(), parsed.resource_bytes())`), so identity and
metadata can no longer come from different resources. `ResourceIdentity::from_bytes` / `::new` added
for the remaining callers; 31 call sites rewritten.

**One guarantee moved, deliberately.** `bind_resource_payload` no longer re-reads the resource slice;
it validates the payload/offset pair and builds the span. A mismatched (identity from A, metadata
from B) pairing cannot be built through `prepare_selected_adm_bwf_input` any more, so the CLI test
that asserted `MetadataPayloadBinding` for a foreign copy now asserts that the identity tracks the
bytes: flipping one PCM byte changes `source_sha256` and the prepared identity. The audit pin
`'MetadataPayloadBinding' in test` became `'resource_identity().input_sha256()' in test`.

Gates: `cargo test --workspace --locked` exit 0, `cargo fmt --check` clean, `check-boundaries.py`
`91 / 104 / 0 / 6`. Goldens: cases 1-15 byte-identical; case 16 is blocked by the flake below.

**Pre-existing flake found while verifying (not caused by this work).** The composed case
(`--dependent-count 1`) intermittently hangs at ~33% of the programme callbacks with ~0 CPU — a
blocked thread, not slow work. It reproduces on a **pristine build** (`git stash` of every change in
this increment, release rebuild) with an emptied loudness cache, and it also completed several times
earlier in the same session, so it is a race in the composed path.
`scripts/golden-vectors.py` now runs each case with a 180 s timeout and names the offender instead of
hanging forever. Worth investigating on its own: `run_worker_with_object_count` spawns a scoped
thread (`ENCODE_REQUEST_STACK_BYTES`) and the composed path drives I0/D0 producers over channels.

Native streaming is therefore **not** switched on yet: the main path at `encode_request.rs:1408` still
uses `read_resource` + resident parse. The streamed preflight experiment was reverted after the flake
made it unverifiable.

**Hang triage so far (2026-11-24, all on the pristine build too):**

- Not the DSP pool: with a temporary `JOCFORGE_NO_DSP_POOL` kill switch forcing the sequential
  fallback, the composed case still hung (reverted afterwards; `dsp_workers.rs` builds the only rayon
  pool and it was the first suspect because the blocked process showed ~14 threads).
- Not the loudness cache lock: `loudness_cache::lock` spins on `try_lock_exclusive` every 80 ms, so a
  bounded wait (20 s, then measure unlocked) was tried. The case still hung well past that bound, so
  the block is not there either (also reverted; the unbounded spin is still a footgun worth bounding
  later).
- With the pool off the blocked process still shows 6 threads, ~0 CPU, wait reasons `UserRequest` /
  `Unknown` — a synchronization wait, not I/O latency.

Next step needs a real stack, not more hypothesis: capture the hung process with `procdump -ma` or
WinDbg and read the wait chain, or add a `std::backtrace::Backtrace::force_capture()` watchdog thread
that prints every N seconds. Both are native-side work.

**Localized further (2026-11-24, tracer since reverted).** With `eprintln!` around
`run_on_worker`'s spawn/join in `application/worker.rs`, a hung composed run printed exactly one
worker pair (`jocforge-main-analysis-cold-root`, spawned and joined) and then the programme callback
progress stalled at 33%. So the block is **not** in the large-stack worker at all: it is inside the
producer call `frames(callback_value)` that `encode_request.rs:1737` performs for callback 4, before
any further worker is spawned. Combined with the earlier results, the stalk is now:
not rayon (`JOCFORGE_NO_DSP_POOL` still hung), not the loudness cache lock (20 s bound exceeded),
not `run_on_worker`. The next probe belongs inside `SpatialFrameProducer::run`'s per-callback path.

Also measured: every composed run (`--dependent-count 1` and `2`) hangs deterministically right now,
while `--data-rate 384/1024` complete in about a second. So the composed path is now a hard blocker
rather than an intermittent flake, which makes it much easier to bisect with a debugger attached.

**Channel handoff traced (2026-11-24, tracer since reverted).** `SpatialFrameProducer::run` on native
runs a producer thread over `mpsc::sync_channel(1)` and the consumer pulls frames. With `eprintln!` on
`produce`/`send`/`take`, a hung run shows:

```
[t] take 3 recv / [t] sending 3 / [t] produced 4
Programme callbacks 33% (4/12)
[t] take 4 recv / [t] sending 4 / [t] produced 5 / [t] sending 5 ... (producer runs ahead)
```

So the producer keeps going and the **consumer blocks inside the callback-4 body**, after receiving
frame 4 and before asking for frame 5. Callbacks 0-3 all logged
`disposition=suppressed`; callback 4 is the first one whose path continues past suppression, so the
block is in the commit/publish half of the loop rather than in frame production.

Raising the lookahead to `sync_channel(4)` did **not** change the stall point, so it is not channel
backpressure: the consumer waits for something the producer cannot provide at any buffer depth.
There are no `Mutex`/`Condvar`/`RwLock` on this path (checked across the workspace outside tests), so
the wait is itself a `recv`/`join` — the next probe should be a stack capture with a debugger
attached to the hung process, since tracing has now ruled out every cheap hypothesis.
