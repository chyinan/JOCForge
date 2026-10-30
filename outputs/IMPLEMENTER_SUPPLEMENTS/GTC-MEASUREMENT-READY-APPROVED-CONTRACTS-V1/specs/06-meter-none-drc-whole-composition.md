# F06 selected full meter 与 none-DRC 组合

状态：`COMPOSITION_RULES_DEFINED / REAL_FULL_CHAIN_FRESH_REPLAY_PENDING`。本条组合已交付的 F06K、F06TP、F06D、OAR meter、F06L、F06I、F06U、F06ND、F06NR、F06NI 和 F06NF。它不重写各模块数值公式。

## 1. 两阶段 programme

完整 programme 分成两个状态隔离的阶段。

1. 独立 meter prepass 从同一 ADM/config 冷启动，完成整首空间渲染和测量。
2. meter finalize 成功后，冻结 programme measurement。主编码从自己的 cold state 开始，逐 source 生成 none-DRC、BSI 和 ID1 输入。

meter 不得读取主编码 audio/history。主编码不得使用滚动 meter prefix。两阶段可以重新读取 ADM，但只能交换最终的有限 measurement scalars、gating kind 和 dialnorm。

## 2. Meter prepass 顺序

模型阶段顺序为：

```text
meter frontend
-> meter OAR render
-> shared DV32
-> valid-audio scheduler
-> true/sample peak and K-energy over the same complete-1024 prefix
-> integrated and ungated mean over each ordered 960-sample energy record
-> getter finalize
```

### 2.1 渲染与有效样本

meter frontend、OAR 和 shared DV32 各自持有冷状态。meter normal callback 对源尾补零，随后执行 3 个 meter zero-drain callback。主前端的 4 个 cached drain 不属于本实例。

空间延迟 3072，加 DV32 延迟 32，总 latency 为 3104。scheduler 去除 startup，并发布 normal callback 域的完整 `normal_callbacks×1536` 样本。只有每路都有完整 1024 样本时，才把同一 block 同时交给 peaks 和 K-filter。EOF 剩余不足 1024 的样本不消费。

### 2.2 Peaks 与 energy

同一完整 1024 block 先后关系不形成数值依赖，允许 peaks 和 K-filter 并行；两者必须消费相同 block identity。

- sample peak 与 true peak 使用 F06TP/F06L；true-peak FIR state 跨 block。
- K-filter 使用六路独立 `2×4` state。滤波结果按 960-sample 窗累计。
- 每完成一个 960 窗，按 `[1/2,1/2,1/2,0,141/200,141/200]` 有序归约。
- 同一个 energy pair 先后送入 integrated 和 ungated mean。它们拥有不同 state，不能互相恢复或覆盖。

EOF 不把 1024 尾或 960 尾补成新记录。finalize 不重新运行 FIR/K-filter，也不 flush true-peak 延迟。

## 3. Getter finalize

最后一个实际 meter callback 成功提交后，按以下顺序读取：

1. true-peak programme maxima；
2. sample-peak programme maxima；
3. integrated result；
4. ungated mean result；
5. DI-off getters：dialogue=-120、speech percentage=-1；
6. loudness selector；
7. dialnorm；
8. 一次提交 immutable programme measurement。字段为 dialogue、speech percentage、ITU、ungated、true peak、sample peak、selected loudness、gating kind 和 dialnorm；按规范 canonical serialization 计算 SHA-256 digest。

当前配置为 BS.1770-4、dialogue intelligence=false、threshold=100。ITU internal 大于 -15/16 时选择 ITU，public gating kind=2；否则选择 ungated，kind=0。true peak 不参与 loudness 选择。

finalize 前任何失败都不生成可供主阶段使用的 measurement。短 programme 尚未产生 integrated event 时，仍可使用已经由 mean writer 产生的 ungated result；两者都未准入时保持 -120 sentinel。

## 4. Main none-DRC 顺序

programme measurement 成功冻结后，main source `s=0..source_count-1` 连续执行：

```text
main frontend installed_audio
-> F07 input history exactly once, producing primary transform_audio
-> same-generation unquantized forward spectrum
-> secondary inverse/overlap reconstruction
-> none-DRC statistics(primary,secondary)
-> none-DRC scheduler/quantizer
-> BSI from final measurement plus same-source DRC
-> ID1 input from the same final measurement
-> source commit
```

primary 是 F07 延迟一次的六路音频。secondary 从同代 primary 的未量化 spectrum 重构，不能使用最终解码 PCM，也不能再做一次 input-history gather。block0 只预热 inverse/overlap；block1..5 映射 secondary block0..4。LFE 和 secondary 最后一块保持正零。

none-DRC 每 source 消费 primary/secondary `6×1536 F32`、同代 short flags 和 programme dialnorm。先完成 statistics，再运行 scheduler。成功返回 RF U16 和 6 个 line U16；F12 只消费各自高 8 位。none profile 仍运行全部统计和 limiter，不能硬编码 RF=FF、line=00。

BSI 和 ID1 均读取同一 frozen measurement digest。每个 main source 的 BSI/ID1 投影记录该 digest；不接受滚动 prefix measurement 或另一个 programme 的 digest。ID1 timer 的 dialogue/ITU/true-peak 输入是 programme final 值；none-DRC 的 per-source peak inputs 与 final true peak 是不同数据流。

## 5. State 与提交

完整 portable schema 见 `../parameters/f06-meter-none-drc-whole-state-schema-v1.json`。三个原子提交点为：

- meter callback commit；
- meter finalize commit；
- main source commit。

每个入口先验证参数、配置、generation、shape 和 finite 域，在 owned working state 中完成全部下游步骤后一次提交。中间失败不提交该原子单元的任何数值 state，随后只提交 lifecycle `{phase=FAILED,failed=true,failure_stage}`。FAILED 状态拒绝所有后续 meter/finalize/main 调用。下一次尝试必须 full reset。

reset 同时重建 meter frontend/OAR、DV32、sample scheduler、K/energy、true peak、integrated、ungated、ordinary history、secondary、none statistics/scheduler 和 override FIFO。不得保留上一个 programme measurement。

完整组合 state 的 canonical recursive encoding、每个 owner 的 cold bits/size/hash 和 leaf schema 路径见 `../parameters/f06-meter-none-drc-whole-state-schema-v1.json` 与 `../test-vectors/F06-METER-NONE-DRC-WHOLE-COLD-STATE-IDENTITIES.json`。失败向量比较全部 owner 的 before/after canonical digest，而非只比较计数器。

Cold identity 的 canonical material 必须先递归把每个 `*_u32_repeat=[count,word]` 替换为同名前缀的完整 `*_u32_words` 数组，再以 sorted keys、逗号/冒号 separators、UTF-8 编码。单 owner hash 直接覆盖该 owner 的展开值；whole hash 覆盖 `owner_name -> expanded cold value` 的 13-owner JSON object，identity 文件自身的 wrapper/schema/摘要字段不进入 hash。不得对 compact repeat descriptor JSON 求 hash，也不得把旧 whole hash 当作 owner hash 的推导输入。

EOF：meter 只执行定义的 3 个 zero drains；main 只消费上游实际提供的 source extent，不加 DRC、secondary、FIR 或 limiter 私有 flush。终态不自动 reset。

## 6. 参数与缺口

selected domain 的数值参数均已有 provider；见 `../test-vectors/F06-METER-NONE-DRC-WHOLE-GAP-INVENTORY.json`。没有发现新的普通常数缺口。当前真实缺口为：

- 没有一个 clean whole composition 实现连接全部模块；
- meter frontend/OAR/DV32 尚无真实 ADM 全程 fresh replay；
- 跨模块失败 transaction 尚无 whole-chain 故障注入。

其他项属于 fresh replay gate：当前 providers 同进程执行、实际 secondary 整曲、完整 getter finalize、4249 source none-DRC、ID1/BSI 同值绑定、reset/EOF。

### 6.1 规范文件映射

- F06OC：`specs/06-oar-model-callback-state-and-clock.md`。
- F06OF：`specs/06-oar-frontend-input-and-clock-join.md`。
- F06M：`specs/06-oar-mix-ramp-and-filter-gate.md`。
- F06OP：`specs/06-oar-point-target.md`。

这些是规范映射，不是运行时 source identifier。

## 7. 合成验收

`../test-vectors/F06-METER-NONE-DRC-WHOLE-INPUT-VECTORS.json` 不含 PCM，只驱动调用顺序和 generation。四条序列共20步，覆盖成功的 meter→finalize→3 main sources、secondary 失败进入 FAILED、FAILED 后拒绝、full reset 后 callback0 重启、独立 reset，以及 rolling measurement digest 拒绝。`../test-vectors/F06-METER-NONE-DRC-WHOLE-VERIFICATION-RESULT.json` 从 cold actual state 连续推进，不使用 expected state seed。成功 source 的 BSI 和 ID1 event 必须携带完全相同的 frozen measurement digest。

这些向量关闭组合状态机，不关闭任一 DSP 数值叶或真实音频验收。

## 8. 独立局部审查

r4 独立审查已通过。审查从 clean freeze 重新展开 14 个 repeat 字段和 43,102 个 U32 word。13 个 owner 及 88,191-byte whole cold identity 全部匹配。4 条序列共 20 步，两个 fatal 行均保持 12 个 numeric owner 不变；9 个 provider 的身份也全部匹配。r3 的 HIGH finding 已关闭。

状态记录见 `../test-vectors/F06-METER-NONE-DRC-WHOLE-LOCAL-REVIEW-STATUS.json`。本结论只使 F06WH 局部范围为 `IMPLEMENTATION_READY=YES`。整曲、render、听感和本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。

## F06WH.10 GTC shared-analysis consumer入口

GTC中F06WH按consumer入口拆分：`SharedAnalysisPrepare`唯一推进F07并交付immutable primary spectrum、installed audio、short flags与analysis after receipt；F06 owner仅执行自己的forward/inverse/secondary、statistics和scheduler。whole-composer入口仅用于F06WH局部验收，不能与F12SC whole-composer在同一source串行调用。F06失败丢弃drc clone和共享analysis clone，二者committed generation均不变。
