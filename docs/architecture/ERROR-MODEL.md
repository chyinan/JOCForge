# 错误模型

当前可执行路径仍只有 domain 的 BackendUnavailable 和 CLI 的 invalid command。帮助/版本退出 0，编码不可用或无效参数退出 2。错误为小写、可行动的片段。输入路径和输出路径目前完全不被解析或触碰。

批准错误分类包括 `UNSUPPORTED_PROFILE`、输入/shape、参数身份、时钟/generation、容量、候选不可行、无可接受候选、queue transaction、完整性、durable I/O 和 programme retired。后续 domain API 必须用枚举承载规范条件和类型化上下文；I/O 错误由 shell 附路径和底层原因。application 保留错误类别。

`UNSUPPORTED_PROFILE` 在入口拒绝未登记组合，且不得创建 staged state。candidate scratch 的不可行结果只在存在后续合法 candidate 时可继续。其余 fatal 必须记录 first failing stage，后续 stage 为 `NOT_RUN`，programme 进入 `RETIRED`。禁止把错误转换成成功空输出，也禁止失败后切换其他算法。

F05R56 producer errors retain the selected contract labels `R56T_STATE_ERROR`, `R56T_PIPELINE_RETIRED`, `R56T_CALLBACK_ORDER`, `R56T_INPUT_ERROR`, `R56T_PROFILE_UNSUPPORTED`, and `R56T_NUMERIC_FAILURE`. The first failing stage wins. Each fatal outcome returns no target payload, receipt, state proposal, or commit and preserves committed state bytes; recovery is a full cold reset. Receipt consumers have a separate ordered `READER_TARGET_RECEIPT_*` validation contract and must fail before producing downstream output; that consumer validation is not part of this producer slice.

发布规则已经由 GTC 和 F15P 定义：所有校验完成前不替换目标；短写循环必须达到 4,096 bytes；零进展、readback、durability 或 replace 失败均为 fatal。失败 attempt 的 temp artifact 标为 abandoned。同一 attempt 不得通过 truncate 后继续。当前仍没有发布实现。
