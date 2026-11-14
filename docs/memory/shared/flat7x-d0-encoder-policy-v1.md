---
status: provisional
topic: "Flat7x D0 four-channel audio policy"
spec: "ETSI TS 102 366 V1.4.1; ETSI TS 103 420 V1.2.1"
confidence: "E-AC-3 syntax verified; source projection and encoding choices provisional"
tags: [JOCForge, Flat7x, D0, E-AC-3, ETSI]
---

# Flat7x D0 四声道音频策略

## 已封存的实现契约

Wash 为实现端封存了 D0 四声道编码契约：[CONTRACT.md](../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1/CONTRACT.md)。Manifest SHA-256 为 689d5e4f3b27fd79110f6fb99189d4407384424929b3a74d20ba3447698f545b，package seal SHA-256 为 5f0c525745784e7b41714d04e5dc432fc2689ccf6ae74e925afe3cf7662e82b6。普通模式和 python -O checker 均通过。

该契约让实现 Agent 继续 D0 producer。D0 coded channels 为 4 个，顺序是 Ls、Rs、Lrs、Rrs；JOC reconstruction carriers 仍为 7 个。ADM source-role map 暂按可替换假设使用，并通过 typed receipt 传递。

D0 音频策略采用 ETSI 允许的 6×256 样本 block 结构、512 点长变换、D15 指数和 ETSI bit-allocation/mantissa 语法。编码器状态按 source generation 独立持有四路 overlap/history；I0、D0、F13 必须绑定同一 AU 后原子提交。EOF 尾部补零并记录 valid/padded sample 数。

D0 目标使用 chbwcod=60，对应 253 个 active bins。隔离仓库已有 F12A BAP provider 限定为 217 个 full-band bins 加 LFE7，且其分发范围标记为 NOT_AUTHORIZED。因此新代码需新增 D0 专用 4×253、无 LFE provider；5.1 provider 保持原样。

## 当前进度

编码算法代码、输入向量执行、DEE/encoder run 和 .ec3 输出均为 NOT_RUN。真实编码仍需单独 Fresh GO。实现 Agent 已收到该契约，并开始按公开 ETSI 表构建 D0 专用 BAP 路径。

2026-11-14 进度：帧尾策略已物化为 `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits`，从 source-bound D0 receipt 传到 AU/frame composer；生产 writer 写入 `auxdatae=0` 并显式零填剩余 auxbits，CRC2 保持最后 16 位。block 2 唯一 skip-field 和 1–511 字节范围继续由现有 assembler 检查。原始 opaque tail 注入只在 `test-support` 下用于结构复制用例。D0 专用 4×253-bin BAP table provider 和 6×4 typed BAP row allocator 已实现，ETSI table-set SHA-256 为 `5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe`。PCM transform、D15 exponent production、mantissa/audblk serialization、六块历史、F12/F13/F15 同源提交和真实输出仍未闭合。

2026-11-14 新增 D0 spectrum slice：`transform_flat7x_d0_pcm_block` 接受四个 512-sample long-window，输出每路 253 个 coefficient bits，并从完整窗口按 ETSI Table 6.33a 计算 `gainrng`。它返回候选频谱，不安装历史，也不生成 exponent、mantissa 或 audblk。5.1 transform 路径保持原样。D15 exponent production/adjustment、mantissa、完整六块 D0 frame 与生产接线仍开放。

2026-11-14 新增 D0 D15 exponent row：按 ETSI clause 7.2.7 从 253 个 coefficient words 提取 raw exponents，并输出 decoder-visible D15 row、U4 seed 和 84 个 U7 groups。Wash 确认 ETSI 不指定唯一 raw-to-D15 preprocessor，并推荐 lower-only 两遍约束。JOCForge V1 策略名为 `JOCFORGE_D15_LOWER_ONLY_SEED15_SLEW2_V1`：seed=min(raw0,15)，正向限制上升不超过2，再反向下调 earlier exponents 使下降不超过2；所有 decoder-visible exponents ≤ raw。4 个 raw spike cases 和 10,000 deterministic rows 验证该模型不变量。它是可替换本地策略，不声称 DEE 行为。 Mantissa normalization、complete audblk、frame-wide fit、owner receipts、CLI 及父级原子提交仍未闭合。


2026-11-14 新增 D0 mantissa、`audfrm` 和 six-block `audblk` 候选。Wash 对 q 选择的最新澄清是 ETSI §7.2.12 的目标为预算内最大已分配比特；raw q 的数值只作确定性并列决胜，不能假设 q-to-cost 单调。JOCForge 使用可替换策略 `JOCFORGE_FLAT7X_D0_MAX_ALLOCATED_BITS_Q_HIGH_TIE_V1`，遍历 q=1023..1，按六块 BAP 和精确 grouped-mantissa bit cost 检查完整 D0 帧，选择最大 fitting content bits，tie 时选较高 q。q=0 保留为合法 zero-BAP 输入向量，产品候选排除它。筛选后的候选再经真实 mantissa 量化、block-two EMDF skip-field、frame tail 和 CRC2 writer；`production_allowed=false`。目前输入仍是调用方提供的六块频谱/D15 rows/BSI/EMDF；PCM history、same-source receipts、CLI 和 F15 原子提交未接通。实际编码需要 Wash Fresh GO。

2026-11-14 新增 `codec::flat7x_d0_pcm_source::stage_flat7x_d0_pcm_block_series` 和 `produce_flat7x_d0_frame_from_pcm_source`：四条 D0 PCM rows 按六个连续 256-sample 新样本块切分；block 0 使用调用方 overlap history，后续 block 使用前一 block 的新样本作为 preceding 256 samples；每块生成 spectrum 与四路 D15 rows，再进入现有 D0 BAP/q-search/audblk/frame composer。完整 4,096-byte frame candidate 和末尾 256-sample history candidate 作为一个纯值返回。programme/attempt/source-generation、valid/padded counts、history 安装、I0/D0/F13/F12/F15 同源提交和 CLI/output caller 仍未接通。

`application::flat7x_pair_source::produce_flat7x_pair_d0_pcm_frame_candidate` 现从 `Flat7xPairSourceBinding` 读取绑定的 D0 rows，并将 D0 source identity 与 role-map receipt 保留在 frame/history candidate 外层。它不安装 history 或提交任何 owner；programme generation、suppressed-source priming、sample extents、F12/F13/F15 join 与 CLI/output caller 仍是下一层接缝。

2026-11-14 新增模型级 F12 pair receipt seam：`Flat7xIntegrityPairRatePlan::flat7x_i0_640_d0_1024` 固定 I0/D0 role-specific validator 为 640/1024 kbps；`IntegrityProgrammeOwner::stage_flat7x_i0_d0_f12_pair_for_callback` 对 exact 2,560/4,096-byte frames 分别做 CRC16 residual 校验，I0 使用无 F13 payload，D0 验证唯一 protected EMDF cursor，且只返回一份 Integrity after-root/owner receipt。父级仅在 durable F15 pair append 成功后安装该根与 D0 history。它是可替换本地模型路径，非 DEE owner/clock 等价声明；extent、source generation 和实际 production route 仍未闭合。

2026-11-14 新增 `application::flat7x_pair_output::stage_flat7x_pcm_pair_for_f15`：将 typed I0 frame、D0 PCM/frame candidate、role-specific F12 pair proposal、F13 metadata proposal 与显式 `Flat7xSourceGeneration` 汇合为一个 ordered AU append candidate；candidate 同时保留 D0 `after_history` 和 Integrity after-root/owner receipt 的访问路径。它不执行 durable append 或安装任何 owner；只有调用方收到同源 F15 durable receipt 后才能完成单次 parent swap。当前应用集成测试使用 input-only metadata fixture，真实 source extent/timebase 与 production F13/CLI join仍未闭合。

2026-11-14 新增 `application::flat7x_pair_integrity::stage_flat7x_pair_f12_integrity`，将已绑定的 I0/D0 frame candidates 与 F13 MetadataSourceProposal、F14K protected container、source/dispatch/AU lineage 一起送入 role-aware F12 pair validator。D0 EMDF payload cursor从帧装配receipt导出，F13仅绑定D0；随后 F15 adapter构成一条 ordered I0→D0 AU proposal。该集成候选测试使用输入向量测试夹具；CLI caller、source extent/EOF tail、durable append 和 parent install仍未闭合。

2026-11-14 新增 CLI D0 BSI source input binder：`codec_source_bridge::bind_flat7x_d0_bsi_source_inputs` 复用 `bind_selected_bsi_drc_fields`，将同源 F06ND dialnorm/RF code/six line codes 与 validated authored downmix levels 带入 target D0 BSI/frame settings；source identity 与当前 `Flat7xSourceRoleMapReceipt` 再次比对。旧的 dialnorm fallback 和硬编码 compr 已移除。策略 `JOCFORGE_FLAT7X_D0_BSI_SAME_SOURCE_F06ND_AUTHORED_MIX_V1` 仍 `production_allowed=false`。可选 information/addbsi 当前按显式 absence policy 留空；reference CI16/addbsi producer lineage、全输入输出、OpenJOC/W04 均未验证。
