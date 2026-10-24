# Full-band 指数行规范化

本切片实现 F10.2～F10.4 的单行全带宽规范化。输入是调用方提供的 `I16[217]` raw 指数行、span、option、gate 和可选 F09PA 参数。该函数不拥有跨帧状态。

测试现覆盖 6 条输入。4 条 `constant(level)` 使用根包结果。另 2 条使用 F10 补充包的 authored recipe。`mod25` 为 `(7*i+3)%25`。`zigzag24` 先算 `q=i//25`、`t=i%25`，偶数 `q` 取 `t`，奇数 `q` 取 `24-t`。两者都对完整 217 项计算，`raw[0]` 也按公式生成。

recipe 扩展只存在于测试中。它们是作者测试输入，不是自然 exponent producer。测试先按公式重建 217 项，核对 input-only 包中的 I16LE SHA，再调用通用 normalizer，最后比较 434 个 native output word 与 washed expected。Wash 的独立 replay 与当前 spec review 都覆盖这两条 authored recipe，结果为零差异。

根包 r4 review 仍绑定旧版 F10 规格 SHA-256 `eea5ed9833a6ca19d859df21f68f3b08e34407ace846f94e64ac5416957c59fb`。当前规格 SHA-256 为 `a538f0af5c70c8e8f4e4b5212438a1e5abd304c3fb2efa7165617014a55fc5fc`。补充 review 只覆盖两条 authored recipe 的 F10.2～F10.4 复算。它不扩展为全 F10 review。

F10.5 三位差分打包已作为独立纯整数 helper 验证。它不组装整行差分，也不写 base/gain 字段。F10 的 reuse 合并、LFE 规范化和 source-frame 计划仍未完成。全 F10 和全 F09 仍未就绪。
