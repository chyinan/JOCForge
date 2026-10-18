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

CurrentFeatures 接受 `Option<CurrentFeaturesAudioView>` 以便对缺失 capability fail closed；`None` 在任何输出或状态 proposal 前都以 `ReadAudioMissing` 拒绝，即使所有 source 都是 RESET/CARRY。正常和 cached-drain 成功路径必须传入显式 proposal view，并同时传入预期 phase。未来 spatial-audio assembler 只消费 `SpatialAudioPostWindowView`，再与同 callback 的 matrix、destination 和 LFE inputs 组合。render producer、assembler 和 parent transaction 仍未实现；这些 view 不授予本地 commit 权限。

## Reset 与失败

fatal 失败丢弃完整 staged root，并将 programme 标为 `RETIRED`。同一 attempt 不得重试 source。恢复只能从 programme 输入起点执行完整 cold rebuild。未完成 AU、pending payload 或临时文件不得发布。

## Validation API

validation 可以读取最终产物，并调用结构解析、channel-map、topology、frame report 和 W04 检查。validation 不提供 production 参数，也不能被 production crate 依赖。
