# Profile 维度

Production `EncodingProfile` 由六个正交维度组成。Validation 使用独立 `ValidationProfile`。公共 API 不得把 render target 传入 production resolver。

| 维度 | 类型 | 当前唯一允许值 | 拒绝规则 |
|---|---|---|---|
| Authored input | `AuthoredInputProfile` | ADM BWF；PCM24；48 kHz；实际行数随输入携带 | 输入行数必须与 RIFF PCM channel count 和 CHNA declared track count 一致，并满足选定 ADM graph 约束；不补齐、截断或重排。 |
| Core carrier | `CoreCarrierProfile` | 5.1；六 carrier | 其他布局或 carrier 数返回 `UNSUPPORTED_PROFILE`。 |
| JOC downmix | `JocDownmixProfile` | DEE selected mode0；phase1；index3 | 其他 mode、phase 或 index 返回 `UNSUPPORTED_PROFILE`。 |
| Substream topology | `SubstreamTopology` | 1 个 independent；0 个 dependent | multi-independent 或任意 dependent 返回 `UNSUPPORTED_PROFILE`。 |
| Codec rate | `CodecRateProfile` / `SelectedDataRate` | 48 kHz 下显式单值 384、448、576、640、768 或 1,024 kbps；6 blocks；1,536 samples/AU；每档对应一个固定 AU 字节数 | 不在六值表内、multi-token 或无法解析的 rate 返回 `UNSUPPORTED_PROFILE`。其他布局和采样率仍不支持。 |
| EMDF carriage | `EmdfCarriageProfile` | `deployed_compat`（默认）或 `etsi_emdf`（可选） | 未登记模式返回 `UNSUPPORTED_PROFILE`，不得降级。 |
| Validation render target | `ValidationProfile.render_target` | OpenJOC 7.1.4，仅用于 validation | 不参与 production profile resolution。 |

## Supported row

当前支持一组共享其余维度的 production profile row。`EmdfCarriageProfile` 在各 row 内允许两个受审 carriage mode。下表不包含 `RenderTarget`。

| AuthoredInputProfile | CoreCarrierProfile | JocDownmixProfile | SubstreamTopology | CodecRateProfile | EmdfCarriageProfile |
|---|---|---|---|---|---|
| selected ADM BWF / PCM24 / 48 kHz；N 个物理行，10 ≤ N ≤ 256，CHNA=N，10 个 bed + N−10 个 object | 5.1 / 六 carrier | mode0 / phase1 / index3 | single independent / zero dependent | 每个显式单值对应下表：6 blocks / 1,536 samples/AU | `deployed_compat` 或 `etsi_emdf` |

| data_rate（kbps） | selector code | E-AC-3 `frmsiz` code | AU bytes | 33 AU bytes |
|---:|---:|---:|---:|---:|
| 384 | 0 | 767 | 1,536 | 50,688 |
| 448 | 1 | 895 | 1,792 | 59,136 |
| 576 | 2 | 1,151 | 2,304 | 76,032 |
| 640 | 3 | 1,279 | 2,560 | 84,480 |
| 768 | 4 | 1,535 | 3,072 | 101,376 |
| 1,024 | 5 | 2,047 | 4,096 | 135,168 |

The six size/code pairs are bounded to the selected 48 kHz/5.1 profile and Wash's 33-syncframe observations. Only one explicit XML token is admitted. The schema's optional default and multi-token `xs:list` runtime semantics remain unbound and are not enabled. This row expansion does not establish DEE runtime host dispatch for every setting or full end-to-end six-rate output equivalence.

### 输入行数扩展的边界

以上 ADM 行数范围是输入/profile 准入维度，不重写 `APPROVAL.md` 中记录的 38-channel 历史基线，也不表示 end-to-end encoder 已接受 N=10..256。CLI 的选定路径使用动态 `NormalControlAdmInputSnapshot`、PCM cursor 与物理行映射；固定的 `NormalControlPreparedSnapshot`、detached primary projection 与 exact-extent map API 仍分别保留 38-row 约束，不应与 CLI 动态路径混称。

### 输入 N 与回调路径的支持矩阵

这里的 `N` 指 RIFF PCM channel count / CHNA physical-track count。各阶段的容量边界如下；容量可容纳不等于已验证支持。

| 阶段 | 代码中的 N 处理 | 当前边界及证据级别 |
|---|---|---|
| RIFF、job configuration、ADM graph、`PreparedJobInput` | 从实际 WAVE/CHNA count 动态构造行和 graph；ADM graph 固定 10 bed，其他为 object。 | 输入准备形状为 10 ≤ N ≤ 256；这是输入准入，不是完整编码支持。 |
| CLI F01E、PCM cursor、primary row bridge | 动态 snapshot/cursor/map 携带同一 N；N 要与 graph、DBMD 和物理 row map 计数一致。 | 代码路径不要求恰好 38 行；BSI profile projection 也只把 N 作为输入侧字段，未设置 DDP per-call frame-view count。 |
| Meter installed48 renderer 与 MAIN F03W source axis | `R=N−1`；`S=11+R=N+10` 个 active source；source-axis 容量为 48。 | Meter renderer 与 F03W 的当前结构性上限均为 N≤38。N>38 会超过 48-source axis；这是容量拒绝边界，不代表 10..38 的所有 job 已通过端到端验证。 |
| DEE descriptor-bearing Reader | V2 静态 source receipt 记录 native descriptor N gate 1..128、固定 128-slot wrapper/map；per-slot row count 是另一维。新 Wash receipt 绑定 Reader 内部 N 的 source 为 descriptor U16 `+0x0A`，zero-extend 后作为独立参数转发。 | Reader descriptor 与 physical CHNA N 的 owner/generation join仍 `UNBOUND`；不能把 descriptor N 或 gate N 自动等同 authored physical N。V2 仅 supersede V1 的 source manifest receipt，保留 V1 包。 |
| Reader PCM allocation | 静态 Reader Pcm extent 为 N×S；N 和 sample-count S 作为两个独立字段/参数传给 allocation helper。S 由 Reader-state selector `+0x14` 选择：zero 时来自 float-read 返回 count，否则来自 Reader-state U32 `+0x10` zero-extend。capacity 以元素增长，之后存在 4-byte scaling overflow path。 | N/S runtime values、selector branch、allocator result 和 overflow outcome 未捕获；physical CHNA N→Reader 与 Reader descriptor→mode-zero composite K same-owner/generation joins仍 `UNBOUND`，不能据此证明 128 行物理输入可用。 |
| Atmos frame-buffer capacity | `+0x68` 是 one-time allocation flag，matrix/table pointers 在 `+0x80/+0x88`。 | 未观察 stored capacity 或 consumer guard；changed-count owner reuse 未捕获，没有安全扩容或超限行为结论。 |
| K 与 composite B | K 使用 U16 count 与六字节 rows；静态 U→K writer path 已绑定；B count 有条件分支。CompositeFormatCount 在所检查路径立即来自 per-source shape metadata/view 或 map transform，而不是该路径中对物理 N 的直接 load/copy。 | runtime U owner/generation 未捕获；physical N→K/B 和实际 B branch 仍 `UNBOUND`。该 scoped first-source 结论不证明其他路径永远无关联。不同 count 域不可互换。 |

当前可报告的只有代码容量边界：CLI/回调各阶段的动态物理行形状最多到 N=38。N=36 与 N=38 是两份独立短窗观察，不建立其间连续范围、一般范围或完整输出支持。ADM/PCM N 到 DDP native frame-view count 的 same-owner/generation join 仍为 `UNBOUND`，不得把 ADM row count 传入 native frame-view count。

### Count-domain separation

下表中的数值域不得因为名称相似或样本值相同而合并。`N` 是输入物理轨数；`K_count`、`CompositeFormatCount`、per-slot physical row slot 和 Reader sample-count `S` 分别有自己的生产者及未闭合边界。

| Typed domain | 当前 Wash/代码证据 | 与其他域的连接 |
|---|---|---|
| Physical ADM `N` | RIFF fmt / CHNA physical rows 在 Rust 输入准备层动态配对；DEE descriptor 路径静态有 U32 `1..=128` gate 和固定 128-slot wrapper/map。 | 物理 N→具体 U/K/B/Reader owner-generation 仍未闭合；静态 gate 不构成产品范围。 |
| `K_count` | K count helper 汇总 individual+group count，结果窄化为 U16 写 K header；row allocation 是 `K_count × 6`。选中 U→K writer 静态在 writer entry 后初始化 U owner slot，再传给 helper。 | 静态 U→K writer path bound；runtime owner/generation、实际 allocation extent、upstream slot validation 与 physical N→K 仍 UNBOUND。 |
| `CompositeFormatCount` | 所检查 composite path 的 immediate source 是 per-source shape metadata/view 或 map transform；nonzero-layout 分支也是 shape-derived。 | 本检查路径没有 physical N 的直接 load/copy 或 typed N→shape join；这不证明全局绝无其他关系。 |
| Per-slot physical row slot | individual writer 用 UID-derived physical slot 写 K row；V2 source receipt 只更正 Ghidra runner postcondition，12-byte accessor仍无本地 bounds check，也不读取 slot+0x4C/K_count。 | 上游 slot validation、实际 allocated extent 与 slot invariant UNBOUND；不据此声称发生 runtime OOB。V1/V2对应同一 accessor finding，只计一条 active evidence slice。 |
| Reader descriptor `N` | getter 从 inner-state `+0x58` 指向的 Reader descriptor 读取 U16 `+0x0A` 并 zero-extend 到 Reader record。selected initializer 将 inner-state `+0x58` 置零；CompositeMaster 的 64-byte source rows 虽复制到 inner state，但所选构造链没有 typed write 给该 descriptor pointer。 | getter 读到的 N 不是已证明的 authored N producer；后续 nonzero writer、physical CHNA `N`→Reader descriptor 和 descriptor→mode-zero K same-owner/generation 均 UNBOUND。 |
| Reader sample-count `S` | Reader-state byte `+0x14` 选择 float-read 返回 count，或 Reader-state U32 `+0x10` zero-extend；`S` 与 Reader `N` 分开传给 allocation helper。 | selected branch、runtime N/S、allocator result 和 Reader→mode-zero composite K same-owner/generation 均未捕获/未闭合。 |

## N / options / algorithm support matrix

本矩阵区分代码准入/容量与Wash 静态路径证据；这些证据不单独构成成功运行、语义效果或端到端支持声明。

| 轴 | 已绑定事实 | 当前未闭合边界 |
|---|---|---|
| ADM physical N | Rust 输入准备动态处理 N=10..256；F03W/Meter 的 48-source 结构容量限制当前回调形状至 N≤38。 | DEE static N=1..128 gate 未与 physical N、K/B 和 Reader/D count 做 same-owner/generation join；无 1..128 E2E 支持结论。 |
| Reader N / sample-count S | Reader getter读 descriptor U16 `+0x0A`，Reader 内部将 N/S 分别传到 PCM allocation；S 由 `+0x14` selector 选 float-read count 或 state U32 `+0x10`。 | descriptor count writer、physical CHNA N→Reader descriptor、Reader→mode-zero K owner/generation、runtime branch和allocation result均 `UNBOUND/NOT_CAPTURED`。 |
| CompositeFormatCount / K_count / row slot | inspected composite path 的 immediate count source 是 per-source shape metadata/map；K 是 U16 count、`K_count×6` allocation；individual writer 使用 UID-derived slot。 | shape count 到 physical N 的同路径 typed join未见；upstream slot validation、allocated extent与slot invariant仍open。没有runtime OOB结论。 |
| `time_base` option | filter key→DDP class及host callback→descriptor→proxy→nested setter静态路径已界定。 | selected XML value/omission到proxy、具体helper目标和descriptor `+0x550` writer仍 `UNBOUND`。 |
| `start` / `end` | XSD optional string/default和独立 Reader range path已分别记录。 | runtime selected值/省略状态、host key/value到Reader实际位置没有typed join。 |
| `offset` / `ffoa` | optional-string schema与static host normalizer已界定；literal `auto` 经converter `stod` 被拒绝。 | selected XML值、host→PrivContext与Reader seek仍 `UNBOUND`。 |
| DRC `A0` row | selected helper把row pointer opaque-forward给converter；converter首读是row `+0x10` QWORD，下一处直接用途与显式零比较；在该 bounded call 中无符号 `QWORD < 0` 的taken条件为false。 | 字段语义及QWORD到snapshot `+0xA0` U32的owner/projection关系没有绑定；trace止于分支目标 callsite，不推断全局不可达、后续转换或DRC/codec/BSI字段。 |
| Rate / RF / DI | rate nested U32通过generic child投影；RF F98 caller/argument edges已静态观察；DI Float32非零分支从subcontext `+0x3C`读取，初值`-120.0`独立于Bool `+0x164`。另一个 DDP encoder EXE candidate也读+0x164，但owner是其EXE child record。 | nested owner/type与BSI、RF candidate→producer vector owner、alternate EXE child→Atmos DLL subcontext、Bool→Float32 result因果均 `UNBOUND`；DI zero-branch无store，不推断其值。 |
| Third-U32 capacity-adjustment capture | E-135 排期独立 scope；E-138 一次性 GO 经 E-140 限定纯 keyless。R141/R1-R4 compatibility 与一条 `+0xB18` candidate-store caller 边界已分别 Wash。 | `NO_GO/BLOCKED_BEFORE_LAUNCH` 保持：没有已确认的 license-free DEE job，E-142 NO-GO 仍有效。R141 record `+0x08` 的 U16 `0x0010` width-five 是 BSID，不是 descriptor `+0x0B1C` third U32；R1/R4 serializer callsites 不匹配 plan callers A/B 且无必需 owner/generation tokens。`FUN_1800d24e0` 的 `[R8+0xB18]` store 仅绑定到一个 caller setup；其 R8 owner/generation、与 Plan-B `*local_68` / `+0xB1C` 的 join、以及它是否为 first writer 均未绑定/未成立。R138 caller applicability `NOT_EVALUATED`。Capture/runtime `NOT_RUN`，third-U32 semantic owner/carrier transfer `UNBOUND`。这是 workflow/evidence blocker，不是 audio/encoder failure。 |

## ValidationProfile

`ValidationProfile` 只包含 `render_target`。改变 `RenderTarget` 不得改变 production `EncodingProfile` 的接受或拒绝结果。

未列入此表的任何组合都返回 `UNSUPPORTED_PROFILE`。不得自动降级，不得切换旧实现，也不得提前加入未来 variant。
