# 模块映射

| 路径 | 职责边界 | 当前实现 |
|---|---|---|
| domain/numeric | 不可变数值类型及基础运算 | 空壳；无常数和 DSP |
| domain/state | 共享不可变契约的归属与错误类型 | 仅 BackendUnavailable |
| domain/frontend | 输入语义验证与规范化 | 空壳 |
| domain/spatial | 空间领域转换 | 空壳 |
| domain/joc | JOC 领域数据与受审规则 | 空壳 |
| domain/meter | 计量规则 | 空壳 |
| domain/transform | 变换领域规则 | 空壳 |
| domain/codec | 音频编码与 wire 职责边界 | 空壳 |
| domain/timeline | F01/F12 时钟、排队和最终 drain 契约 | 空壳；未选择帧长 |
| domain/publication | 不可变待发布结果与声明 | 空壳 |
| domain/transaction | 纯提交/撤销决策 | 空壳 |
| application | 验证请求并协调各 owner | 仅拒绝 encode |
| io/bw64、pcm | 有界字节、chunk 和 PCM 读取适配 | 空壳；未复制旧扫描器 |
| io/atomic_output | 临时写入、flush、rename 和回滚副作用 | 空壳 |
| CLI | 参数、退出码、未来 I/O 协调 | help/version/拒绝编码 |
| validation | 独立解码/比较工具适配 | 空壳；无旧验收门 |

以上为职责划分，不能作为 GTC owner 映射或算法授权。当前无未来 profile 抽象、插件或 feature 生产分支。函数签名应等规范后按实际需要建立。
