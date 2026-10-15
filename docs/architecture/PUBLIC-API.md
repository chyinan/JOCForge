# 公共 API 边界

本轮只定义类型和职责，不定义 Rust 函数实现。

## Profile API

调用者必须提交七个独立 profile 维度。入口先验证完整组合。只有 `PROFILE-DIMENSIONS.md` 的 supported row 可进入后续协调。拒绝结果为类型化 `UNSUPPORTED_PROFILE`，不得产生 staged state、临时 AU 或输出文件。

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

application 只协调 receipt。它不得计算 FFT、QMF、BAP、mantissa、Huffman、payload 或 frame。CLI 只负责参数和 I/O 组装。validation 不进入 production 依赖图。

## Reset 与失败

fatal 失败丢弃完整 staged root，并将 programme 标为 `RETIRED`。同一 attempt 不得重试 source。恢复只能从 programme 输入起点执行完整 cold rebuild。未完成 AU、pending payload 或临时文件不得发布。

## Validation API

validation 可以读取最终产物，并调用结构解析、channel-map、topology、frame report 和 W04 检查。validation 不提供 production 参数，也不能被 production crate 依赖。
