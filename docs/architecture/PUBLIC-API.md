# 公共 API 边界

架构层定义类型和职责；各 direct slice 的具体函数签名以其批准 machine map 为准。

## Profile API

Production 调用者提交六维 `EncodingProfile`。入口先验证完整组合。只有 `PROFILE-DIMENSIONS.md` 的 supported row 可进入后续协调。拒绝结果为类型化 `UNSUPPORTED_PROFILE`，不得产生 staged state、临时 AU 或输出文件。

`RenderTarget` 只存在于独立 `ValidationProfile`。Production encode 不接收 `RenderTarget`，render target 的变化不得影响 production profile resolution。

## 时间域 API

公共接口必须使用不同 newtype：

- `AdmSampleIndex`
- `FrontendCallbackIndex`
- `ReceiverCallbackIndex`
- `CodecSourceIndex`
- `JocDispatchIndex`
- `JocWriterIndex`
- `PublishedAuIndex`

这些类型不得互相隐式转换。公共接口不得用裸 `usize` 表达时间域。跨域转换只能由 `CLOCK-MAP.md` 登记的关系生成，并返回带输入域、输出域和规格身份的 immutable receipt。

## Owner API

owner 的公开边界只接收 immutable input、proposal 或 staged result。每次 stage 返回类型化 owner receipt。receipt 至少携带 owner、transaction、before/after generation、canonical identity、changed 和 commit count。Hash 只验证身份，不能代替完整 state。

完整 owner receipt 元数据只用于 committed state owner 和明确登记的 staged owner boundary。纯 numeric 结果、profile 校验和普通时间转换返回轻量 typed value 或 conversion result，不得为了统一框架包装 heavyweight receipt。

## Metadata owner selected source API

`domain::joc::emdf::metadata_bodies::assemble_metadata_bodies` 把同一次 admitted J05/reader receipt 的 ID11、由该 ID11 派生的 ID2 和已成功的 ID14 绑定到同一个 programme/attempt/transaction/source/dispatch。`MetadataOwner::stage_source` 只接受该 lineage-bearing proposal、typed `Id1MeasurementConfiguration` 和 `PublishedAuIndex`；wrapper configuration 由 constructor 固定，不在每个 source 重新传入。返回 `MetadataSourceProposal`，其中含 owned after-state、specialized receipt 和可选 EMDF，但没有安装或 programme-retire 权限。

`MetadataSourceCall`/`process_source` 保留现有 leaf vector API 的本地行为；production parent 不得用它替代 `stage_source`。Measurement receipt gating 只接受 exact `UNGATED`/`DIALOGUE`/`LEVEL`，并在类型内保留 public semantic 0/1/2；selected DI-off ID1 adapter拒绝 DIALOGUE。不得从 `preferred_downmix_mode`、internal enum ordinal、大小写 alias 或裸 numeric fixture 推断 config[2]。只有完整 meter prepass/finalize 才能 mint 该 receipt。时间域使用 typed indices，不接受裸 `usize`。

`frontend::selected_adm_graph::parse_selected_adm_graph(axml, chna)` 是纯输入 API：仅接受调用者传入的 AXML 与 CHNA metadata bytes，返回 candidate 38-track graph/ATU UID identity 和物理 bed join receipt；`admit_object_events(source_samples)` 对此candidate解析单block对象字段与selected时钟域，`build_candidate_object_controls(source_samples, dbmd)` 可消费同源解析的DBMD字段。该candidate API不绑定F01E main job、不在主encode调用图内；不接收path/file handle，不读取WAV data/PCM，也不接管mutable callback state。regular source relation使用`CodecSourceIndex`；物理host和一基CHNA track index使用不同newtype。

production stage 成功返回 `MetadataSourceProposal`，不替换 committed metadata root。EOF 只返回 `MetadataEofProposal`，不生成 source receipt、codec flush 或 durable output。specialized receipt 不是 global `OwnerReceipt<Metadata>`；parent 必须在 F12、integrity 和 durable 条件成功后与其他 owner 一次安装。当前产品入口仍只接 `deployed_compat` selected wrapper；F13C/F12 complexity join、`etsi_emdf` product admission 和全局协调仍 deferred。

application 只协调 receipt。它不得计算 FFT、QMF、BAP、mantissa、Huffman、payload 或 frame。CLI 只负责参数和 I/O 组装。validation 不进入 production 依赖图。

## PostWindow fanout capability

`PostWindowRenderProposal` 表示一次 `INSTALLED48_POST_WINDOW_48X3072` render。Future `spatial_control/F03W` renderer 通过内部构造边界创建它；调用者不能提交 raw audio 或默认值。

`fanout(self)` 只返回 `CurrentFeaturesAudioView` 和 `SpatialAudioPostWindowView`。proposal 与两个 view 都不能 `Clone` 或 `Copy`。两个 view 共享同一个只读 `Arc<[FiniteF32]>`，并保留相同 lineage、显式 `CallbackPhase` 和 payload SHA-256。

CurrentFeatures 接受 `Option<CurrentFeaturesAudioView>`。当至少一个 source 为 READ（`00`）时，必须提供 view；否则以 `ReadAudioMissing` 拒绝。全 RESET/CARRY 时，`None` 合法，CurrentFeatures 不读取音频样本，也不调用 FFT。调用者提供 `Some(view)` 时，即使没有 READ source，CurrentFeatures 仍验证 phase、lineage 和 root identity。正常和 cached-drain 成功路径都接收显式预期 phase，不从音频值推断 phase。

PostWindow fanout 仍生成两个 view。若没有 READ source，父级可以丢弃未消费的 `CurrentFeaturesAudioView`，并保留另一个 view 给 assembler。丢弃 view 不会提交 owner state。未来 spatial-audio assembler 只消费 `SpatialAudioPostWindowView`，再与同 callback 的 matrix、destination 和 LFE inputs 组合。render producer、assembler 和 parent transaction 仍未实现；这些 view 不授予本地 commit 权限。

Direct-LFE scoped stage 是 crate-internal F04C seam。`DirectLfeInputView` 只能从 `spatial_control` 的 opaque ingress authority、validated semantic route 和显式 callback phase 构造；stage 返回线性 ring after-proposal 及可选 Type10 row0 input，不生成 committed receipt，也没有 local commit。

P1.3-E4 的 crate-internal Type10 API 消费同 callback 的 E2 `ComponentStageProposal` 和 E3 `LfeStageProposal`。`spatial_audio_composer` 随后以 admitted reader receipt 验证完整 parent/target lineage：g0/g1 只物化 full-root priming after；g>=2 执行 receiver/mode0 fitting 各一次，并返回 canonical `SpatialAudioAfterProposal` 与 immutable `SelectedJocQmfProjection`。projection 保留实际 QMF/target/final-six buffers、component/LFE origin、reader whole-state hashes和16行 producer generation。任何 hash 都不能代替这些值。composer 没有 commit 权限；parent transaction 仍是唯一安装者。

`JocPayloadInput::from_projection` 只能从该 J05 projection 创建。`stage_selected_joc_source` 校验 fit lineage，使用同次 F04D selected index，所有 representation retry 共用一个 sequence；完整 body 成功后返回 `SelectedJocAfterProposal`，其中含 exact `JOCSEL01` after-root、ID14 body 和 lineage。JOCSEL01不包含 F04 state、F13 queues或 coefficient scratch；proposal 没有 local commit API。

## Reset 与失败

fatal 失败丢弃完整 staged root，并将 programme 标为 `RETIRED`。同一 attempt 不得重试 source。恢复只能从 programme 输入起点执行完整 cold rebuild。未完成 AU、pending payload 或临时文件不得发布。

## Validation API

validation 可以读取最终产物，并调用结构解析、channel-map、topology、frame report 和 W04 检查。validation 不提供 production 参数，也不能被 production crate 依赖。

## F05T selected trim-section API

`domain::joc::oamd::trim_section` 暴露纯函数 `encode_trim_body`、`decode_trim_body`、`parse_dbmd_sections` 和 `selected_trim_from_dbmd_sections`，以及 value-only `Trim`、`EncodedTrimBody` 和 `TrimSectionError`。该 direct slice 只覆盖 Type11 trim body 编解码、DBMD section framing 与 sections 9/10 的 selected default-trim projection；它不解析完整 DBMD 语义、不做非默认 trim 量化、不写入 F05O 外层 section，也不拥有 callback、queue 或 programme state。准确输入/输出与错误边界见 `IMPLEMENTATION-MAP.json` 的 `F05T/trim-section-codec` direct-slice 记录。
