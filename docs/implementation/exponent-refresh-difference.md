# F09E D06-E 差值映射

此切片把一个严格的 `i16` 索引映射到 D06-E 表值。函数不维护状态，不访问 provider，也不执行浮点运算。

| 索引 | 输出 |
|---|---:|
| `0..=1` | `0` |
| `2` | `1` |
| `3..=31` | `2 * index - 4` |

索引小于 `0` 或大于 `31` 时，函数返回 `EXPONENT_INPUT_ERROR`。Rust 接口接收 `i16`，所以 bool、浮点、字符串和 null 不属于此函数的输入类型；本切片不新增 JSON 解析接口。

32 个输出按索引升序序列化为 I16LE，共 64 字节。生成摘要为 `a0e75dafa9f3c1627ec36351ed857711c5a79f60fd7fda0b9f988f0505a4a070`。

验收覆盖 32 个有效索引、两个可由 Rust 类型表达的越界值，以及生成表摘要。Wash 包另外记录 5 个非整数 JSON 拒绝向量；这些值不能传给本切片的静态类型接口。

此切片不执行完整 exponent refresh、F10 normalization、BAP allocation、frame planning 或 policy-provider loading。整个 F09/F10 与产品链路仍未就绪。Wash 包的原始 `INTERNAL_ONLY` 声明保留；仓库发布授权见 [PUBLIC-DISTRIBUTION-AUTHORIZATION.md](PUBLIC-DISTRIBUTION-AUTHORIZATION.md)。
