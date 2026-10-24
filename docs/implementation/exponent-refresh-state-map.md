# F09 exponent refresh state helper

本切片实现一个有界的 in-place refresh helper。函数更新调用方提供的 `current`、`work` 和 `marker`；完整字段顺序见 [`CONTRACT.md`](../../crates/jocforge-domain/tests/fixtures/exponent_refresh_state/CONTRACT.md)。

调用方必须满足行长度、索引范围、selector 域和行值域等前置条件。函数不校验这些条件，也不返回状态码。前置条件不成立时，行为未定义于洗白包；实现不新增错误映射。

`mode` 和 `special` 使用 `i16`。这保留了包中记录的 16-bit 操作数，不把 Wash 提出的 `bool` 或 enum 选择说成 native typedef。函数不拥有 callback/source/AU 时钟，也不执行 reset、EOF、fatal 或 parent rollback。

验收用 10 个 input recipes 重放 12 次连续状态转换，逐步比较 `current`、`work` 和 `marker`。normal/special threshold discriminator、marker precedence、weighted/ordinary 分支和连续调用都由 vectors 覆盖。

此 slice 只覆盖 refresh helper。F09 上游 POWER/bias/config producer、完整 F09 lifecycle，以及 F10/F11/F12 和整条音频链路仍未完成。原始 Wash `INTERNAL_IMPLEMENTATION_HANDOFF` 范围保持不变；仓库公开发布授权单独记录于 [PUBLIC-DISTRIBUTION-AUTHORIZATION.md](PUBLIC-DISTRIBUTION-AUTHORIZATION.md)。
