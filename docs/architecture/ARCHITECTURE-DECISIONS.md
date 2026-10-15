# 架构决策

## AD-001 独立物化批准设计

本仓库不继承 `469cc61cb4345c89dc96cb205f991ac23c26457b` 的 Git 历史。该值只作为批准设计参考身份。当前文件从已核验的 IMPLEMENTER_PACKAGE 独立物化，不读取或复制旧仓库源码。

## AD-002 只支持 selected profile

七个 profile 维度必须显式建模。当前只登记 `PROFILE-DIMENSIONS.md` 中的一行。其他组合返回 `UNSUPPORTED_PROFILE`。不设计未来 profile 框架，不提供自动降级或 fallback。

## AD-003 唯一状态 owner

批准的 owner projection 有 7 个 field group、7 个 receipt owner 和 6 个 committed owner。`body_proposals` 是纯值，无 committed state。每个持久字段只能由一个 owner stage 和 commit。

## AD-004 类型化时钟

ADM sample、frontend callback、receiver callback、codec source、JOC dispatch、JOC writer 和 published AU 使用不同 newtype。公共接口禁止裸 `usize` 时间索引。

## AD-005 staged root 与 fail-stop

所有可变 owner state 都在 attempt 私有 staged root 中推进。fatal 时 committed owner identity 不变，programme 进入 `RETIRED`。恢复执行 programme-origin full restart，不声明旧实现具有全局 rollback。

## AD-006 production 与 validation 分离

production 不依赖 validation。E-AC-3 structural parser、channel-map validator、topology validator、frame report 和 W04 只存在于 validation crate。

## AD-007 公共 wire 原语重新实现

允许的公共 wire 能力以 `OLD-WIRE-REUSE-MATRIX.md` 为准。所有能力按批准规格重新实现。不得恢复旧 writer、serializer、producer、rate plan 或 expected bytes。

## AD-008 keyless publication

私有厂商 key 不在产品范围。F14K 只产生当前 frame 的公共完整性 receipt。F15 是 staging stream、durable ledger 和发布计数的唯一 commit owner。
