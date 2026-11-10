# 架构决策

## AD-001 独立物化批准设计

本仓库不继承 `469cc61cb4345c89dc96cb205f991ac23c26457b` 的 Git 历史。该值只作为批准设计参考身份。当前文件从已核验的 IMPLEMENTER_PACKAGE 独立物化，不读取或复制旧仓库源码。

## AD-002 当前 production profile 与 additive contract 分离

Rust `EncodingProfile` 现在以七个 immutable fields 表达现有六个 profile dimensions 加 reconstruction layout。既有 5.1 rows 保持原解析与 header bytes；1024 kbps / 5.1 core 的 Flat7x partial row 由新 typed layout 选择。另一个已批准的 Atmos 7.1 flat7x / 1664 I0/D0 row 是独立 AU contract，完整输出验收仍为 `NOT_RUN`。Profile resolution 不做自动 fallback。

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

私有厂商 key 不在产品范围。F14K 只支持公开 keyless CRC profile。integrity owner 保留 profile/provider identity、source 进度、published AU count 和 finalized 状态。每次调用依次执行 SOURCE、DISPATCH、AU，并在 AU 校验成功后提交一次本地 owner state。

integrity owner 的局部提交不发布 frame，也不代表全局 CALLBACK commit。F15 仍是 staging stream、durable ledger 和已发布 AU 计数的输出 commit owner。私有认证 profile 和 native receipt 格式保持 blocked。

## AD-009 integrity owner 与 callback 协调分离

keyless integrity root 先作为独立 direct slice 验证。它返回 SOURCE、DISPATCH、AU 三个 provisional stage receipts，并在成功后更新本地 owner state。任何失败都丢弃所有临时输出，保持 committed state 不变，并将 attempt 标记为 `RETIRED`。

此 slice 不接入 CallbackTransaction。未来 coordinator 仍须在 12 个 owner roots 和所需 source-stage roots 完整后，统一提交 CALLBACK receipt。当前局部 commit count 不等于 global top-level commit count。

## AD-010 additive Atmos 7.1 target contract

The sealed `JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1` defines one additive target row. It preserves the existing 5.1 row, native I0 path, multirate choices, and output identity; it does not enable production output or automatic fallback.

The legacy `SelectedMode0Phase1Index3` value combines distinct meanings. For the contract, downmix mode, algorithm phase control, and the contract-level downmix-index candidate are separate from `JocReconstructionProfile`. The 5.1 receipts independently show mode0/phase-flag1, derived downmix_index 3, and a separate full-file reconstruction profile `five_x_phase`/index3. Equal numeric 3 does not establish typed identity; the legacy enum's `Index3` serializer relation remains `UNBOUND`. The DDP XML `preferred_downmix_mode` MixMode is also separate, with effective storage and relation to reconstruction profile `UNBOUND`. The target reconstruction is `flat7x`/index1. Never interpret profile index as mode/phase, or wire `phase_signaling` as the algorithm phase flag; target mode/phase controls remain `UNBOUND`.

The target access unit contains ordered I0 then D0 receipts from the same source generation. F12 supplies separate I0/D0 frames; F13 binds one complete EMDF container to D0 audio-block skipfld block 2; an integrity receipt is required; F15 publishes the whole AU atomically. A missing frame/role/metadata/integrity receipt or a container above 511 bytes rejects the entire AU; I0-only publication is forbidden. GTC source-role projection, native D0 audio algorithm, and all-AU EMDF fit remain `UNBOUND/NOT_RUN`.

The local `Flat7xI0D0TargetPlan`, candidate validator and byte-sequence composer model this contract outside `resolve_selected_profile`. The composer concatenates CRC-checked I0 and D0 byte slices in target order; it does not parse their syntax. An additive F15 target owner stages one complete 6,656-byte AU proposal and pair-aware ledger entry. The separate `jocforge-io::flat7x_au_append` adapter can write that proposal to a caller-owned staging file, sync it and verify readback before returning a caller-supplied receipt candidate. This adapter does not roll back partial staging writes, finalize or expose a destination, register with GTC, or connect to the selected 5.1 output sink. The target retains `production_allowed=false`.

## AD-011 Flat7x reconstruction layout as immutable profile configuration

The selected 1024 kbps / 5.1 core path gains the optional CLI selector `--joc-layout flat7x` and the separate `JocReconstructionLayout::Flat7x` value, resolved to ETSI `joc_dmx_config_idx=1` in the 3-bit JOC header field. Omitting the selector preserves the current 5.1 profile. `JocDownmixProfile::SelectedMode0Phase1Index3` remains the independent mode0 / phase1 / internal index3 control. The resolved profile passes both values to the existing JOC-header writer boundary; header order remains the layout index, 6-bit object-count field, then 3-bit extension index.

The Flat7x layout is immutable per-attempt profile configuration projected through the existing GTC/profile-resolution boundary. It adds no mutable owner or commit stage. The existing 5.1 profile row and its audio-generation behavior stay stable. This decision covers profile resolution and this header field; complete Flat7x stream structure and official-demo acceptance remain `NOT_RUN`.

The separate 1664 kbps I0/D0 AU contract remains its own target row and publication plan. This 1024 kbps partial slice changes only the reconstruction-layout profile value and its ordered JOC header field.
