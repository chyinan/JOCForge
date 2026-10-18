# 公共 API 边界

本轮只定义类型和职责，不定义 Rust 函数实现。

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

application 只协调 receipt。它不得计算 FFT、QMF、BAP、mantissa、Huffman、payload 或 frame。CLI 只负责参数和 I/O 组装。validation 不进入 production 依赖图。

## PostWindow fanout capability

`PostWindowRenderProposal` 表示一次 `INSTALLED48_POST_WINDOW_48X3072` render。Future `spatial_control/F03W` renderer 通过内部构造边界创建它；调用者不能提交 raw audio 或默认值。

`fanout(self)` 只返回 `CurrentFeaturesAudioView` 和 `SpatialAudioPostWindowView`。proposal 与两个 view 都不能 `Clone` 或 `Copy`。两个 view 共享同一个只读 `Arc<[FiniteF32]>`，并保留相同 lineage、显式 `CallbackPhase` 和 payload SHA-256。

CurrentFeatures 接受 `Option<CurrentFeaturesAudioView>`。当至少一个 source 为 READ（`00`）时，必须提供 view；否则以 `ReadAudioMissing` 拒绝。全 RESET/CARRY 时，`None` 合法，CurrentFeatures 不读取音频样本，也不调用 FFT。调用者提供 `Some(view)` 时，即使没有 READ source，CurrentFeatures 仍验证 phase、lineage 和 root identity。正常和 cached-drain 成功路径都接收显式预期 phase，不从音频值推断 phase。

PostWindow fanout 仍生成两个 view。若没有 READ source，父级可以丢弃未消费的 `CurrentFeaturesAudioView`，并保留另一个 view 给 assembler。丢弃 view 不会提交 owner state。未来 spatial-audio assembler 只消费 `SpatialAudioPostWindowView`，再与同 callback 的 matrix、destination 和 LFE inputs 组合。render producer、assembler 和 parent transaction 仍未实现；这些 view 不授予本地 commit 权限。

Direct-LFE scoped stage 是 crate-internal F04C seam。`DirectLfeInputView` 只能从 `spatial_control` 的 opaque ingress authority、validated semantic route 和显式 callback phase 构造；stage 返回线性 ring after-proposal 及可选 Type10 row0 input，不构造完整 Type10、不生成 committed receipt，也没有 local commit。

## Reset 与失败

fatal 失败丢弃完整 staged root，并将 programme 标为 `RETIRED`。同一 attempt 不得重试 source。恢复只能从 programme 输入起点执行完整 cold rebuild。未完成 AU、pending payload 或临时文件不得发布。

## Validation API

validation 可以读取最终产物，并调用结构解析、channel-map、topology、frame report 和 W04 检查。validation 不提供 production 参数，也不能被 production crate 依赖。
