# 公共 wire 复用矩阵

本矩阵只适用于当前批准的 selected profile。所有条目都必须按权威规格重新实现。不得复制旧代码、旧 expected bytes 或旧兼容路径。

## 可在 production 重新实现

| 能力 | 边界 |
|---|---|
| bounded MSB bit I/O | 必须检查 cursor 和容量，失败时零提交。 |
| CRC16 | 只实现权威规格指定的多项式、初值、处理顺序和 residual 检查。 |
| frame-size words-minus-one | 只接受 selected profile 的 4,096-byte AU。 |
| selected header/schema | 只实现 selected profile，保留码必须拒绝。 |
| checked capacity | 所有长度、偏移和位数运算必须检查溢出。 |
| Huffman tree traversal | 表和遍历规则只能来自权威规格包。 |
| 公共结构校验 | 只校验公共结构，不推导音频、JOC 或空间正确性。 |

## 只允许进入 validation

- E-AC-3 structural parser。
- channel-map validator。
- topology validator。
- frame report。

这些能力不得被 production 调用，也不得生成 production bitstream。

## 禁止恢复

- 旧 writer。
- 旧 EMDF serializer。
- 旧 OAMD/JOC producer。
- 旧 BAP/mantissa producer。
- FFmpeg skip-field 注入。
- legacy/multi-dependent writer。
- 旧 profile resolver。
- 旧 rate plan。
- 旧 expected bytes。

任何未列入“可在 production 重新实现”的能力，默认不允许从旧实现复用。遇到规格缺口时必须报告 blocker，不得猜测或恢复旧 fallback。
