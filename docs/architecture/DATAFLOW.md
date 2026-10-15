# 数据流

当前可执行调用图：

```mermaid
flowchart TD
  M[CLI main] --> H[help / version: exit 0]
  M --> E[encode]
  E --> A[application::encode]
  A --> D[domain::state::EncodeError::BackendUnavailable]
  D --> R[stderr / exit 2]
  M --> U[unknown command: exit 2]
```

encode 不读取路径、不启动进程、不打开文件。其余 domain 模块、io 和 validation 均不在当前调用图中。

批准后的职责流：CLI 收集有界输入 → io 产生输入字节 → frontend 生成不可变语义输入 → application 按 GTC 协调 owner → 各 domain 产生 staged result → publication 产生完整发布提案 → CLI 调用 atomic_output 提交。本轮只物化边界，不启用这条生产路径。

## Main 时钟

| 域 | 范围或关系 | 行为 |
|---|---|---|
| normal callback | `g=0..4246` | 读取真实输入；最后一次含 1,050 个真实样本和 486 个正零 padding。 |
| cached drain | `g=4247..4250` | 4 次 cached drain；复用末次完整 padded host38 和 control snapshot。 |
| codec source | `s=g-2`，仅 `g>=2` | 共 4,249 个 source，`s=0..4248`。 |
| suppressed source | `s=0,1` | 完成 source 阶段和 queue pop；不发布 AU。 |
| published AU | `j=s-2`，仅 `s>=2` | 共 4,247 个 AU，`j=0..4246`。 |
| JOC dispatch | `d=s+1=g-1` | source 域有效时建立 dispatch，范围为 `1..4249`。 |
| JOC generated writer | `s` | 每个 source 生成 `JocGeneratedWriterIndex(s)`。 |
| publication pop | `s-1` | source `s` 产生 `JocPoppedWriterIndex(s-1)`；source 1 的 writer 0 标记为 `CONSUMED_SUPPRESSED`。 |
| AU payload join | AU `j` 消费 writer `j` 和 `j+1` | 分别使用 `OrdinaryPayloadWriterIndex(j)` 和 `JocPoppedWriterIndex(j+1)`。 |

EOF 不新增 codec flush。ordinary pending writer `4247` 和 `4248` 丢弃。ID14/JOC pending writer `4248` 丢弃。最终长度为 `4,247 × 4,096 = 17,395,712` bytes。

`g`、`s`、`d`、`j`、JOC generated writer、ordinary payload writer 和 JOC popped writer 分别使用 `FrontendCallbackIndex`、`CodecSourceIndex`、`JocDispatchIndex`、`PublishedAuIndex`、`JocGeneratedWriterIndex`、`OrdinaryPayloadWriterIndex` 和 `JocPoppedWriterIndex`。公共接口不得用裸 `usize` 混用时间域。

独立 validation 读取最终发布结果，调用外部工具解码或比较，输出带工具身份的证据。它不能反馈生产算法常数，不能作为编码器主体。FFmpeg 不属于 production 依赖图。
