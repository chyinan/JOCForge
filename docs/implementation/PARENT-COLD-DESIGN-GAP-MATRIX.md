# Parent cold-state 设计缺口矩阵

## 状态

本文件是实现侧 inventory，不是 canonical schema。它整理 v34 洗白资产和 Wash Agent 的 structured gap response。当前唯一完整 profile 仍是 selected ADM/configuration。Gate 1 必需逆向依赖数为 0。设计和验证未完成前，保持 `IMPLEMENTATION_READY=NO`、`production_allowed=false`。

## 字段和对象 inventory

| 区域 | 已洗白事实 | 当前分类 | 缺口与关闭条件 |
| --- | --- | --- | --- |
| 120-byte cold descriptor | 布局包含 mode、source count、data-rate enum、指针、mode-specific tuple 和 measurement summary。selected run 是 mode 0、source count 38、data-rate code 5。 | `INVENTORY_PARTIAL` | `+24..+39` 的跨 mode 语义、measurement `+8/+12/+16` 名称和引用块全量语义未构成完整逻辑字段表。需逐字段 schema，注明 consumed/excluded/zeroed。 |
| 44-byte policy block | run109 的 16 对 property/value 已闭合；policy `+2`→property31 mixlevel，`+4`→property36 roomtyp，`+32`→property29 adconvtyp。 | `CURRENT_RUN_VALUES_CLOSED` | 仅限已识别 run/config identity。不得移植到其他 profile。需 schema 定义 logical representation 和跨 profile policy。 |
| 38 个 channel records | 每条记录 6 bytes，字段为 input channel key、host index、source ordinal，各为 U16。 | `FIELD_LAYOUT_CLOSED` | 当前适配器只消费 key；raw records 未交付。需定义各字段在 portable parent state 中的使用和排序合同。 |
| 120-byte packed metadata | selected constructor 写入 106 bytes；14 个 gap 是 `78..87`、`99`、`107`、`118..119`。selected init 将它们置零；`80..87` 的 selected-run 最终值来自 ordinal 3 的 binary64 positive zero。 | `SELECTED_RUN_CAUSAL_JOIN_CLOSED` | 只覆盖当前 ADM/configuration 和 composite generation。generic setter 可替换全部 120 bytes，但不能据此定义所有 write path。需有配置范围及完整 logical field inventory。 |
| Packed metadata provider | selected source 是 MasterSourceImpl。source pointer、composite owner `+264`、producer owner `+88` 和 parent `0x178DC0` 的 pointer/owner/order join 已闭合。 | `CURRENT_RUN_IDENTITY_CLOSED` | 当前 source/master 关系不外推。27×U32 dictionary 在 accepted path 未调用，且没有同 owner/generation join；禁止 offset transplant。 |
| External metadata 其他字段 | 120-byte producer、helper offsets 和 selected gap writes 有 bounded evidence；generic setter whole-copy 有独立路径证据。 | `SCHEMA_INCOMPLETE` | 未知字段、constructor owner/first writer 和独立 dictionary join仍未闭合。需字段 owner、值来源和同代 transfer 证据，或保持字段 excluded 并证明逻辑上不需要。 |
| Measurement | core slots 8/12/28 到 public result 12/16/8 的 binary32×128 映射已闭合；public `+20` 的 measured loudness 是已命名字段。 | `PARTIAL_LAYOUT_CLOSED` | public `+8/+12/+16` 名称未闭合。Dolby TrueHD（DTHD）public Meter 是 vptr+PIMPL 的 16-byte wrapper，不能把 PIMPL offsets 当作 DDP public layout。需 named owner/consumer 或明确排除。 |
| Trim payload | 24-byte payload 的 flag、padding、gains、balances、copy 和 consumer permutation 有闭合 evidence。 | `PAYLOAD_LAYOUT_CLOSED` | 不等同于 parent cold-state framing。需决定它如何归入 canonical state、哪些字段被消费，以及 vector 覆盖。 |
| Runtime pointers/PIMPL | selected prefix `+88/+92` 指针在 persistence 前清零；parent runtime allocation 含进程内 pointers 和 nested objects。 | `EXCLUDED_FROM_PORTABLE_BYTES_CANDIDATE` | canonical schema 尚未逐字段确认所有 pointer/opaque/padding 的排除规则。不得序列化地址或把整个 allocation 当作 portable bytes。 |
| Parent host lifecycle | 当前观察到 close/destroy 后为新记录创建 fresh inner object；同对象 restore 未观察到。 | `FRESH_RECONSTRUCTION_OBSERVED` | 这不是 serialization/restore 或 receipt。动态 second run 仍 `NOT_RUN`；需独立验证 token/state retirement 与 fresh reconstruction acceptance。 |
| F01E / optional caches / writer set | framing rules 将 top-level envelope、F01E embedding、optional diffusion/source cache、current flags/current features、writer set 列为缺口或冲突。 | `FRAMING_BLOCKED` | 需每项明确 presence、长度、section、顺序和空/非空编码；禁止 unknown zero-fill。 |

## 设计与验证依赖节点

这些节点按依赖顺序关闭。下游节点不得跳过上游未定义字段。

| 节点 | 状态 | 最小关闭条件 | 未运行验证 |
| --- | --- | --- | --- |
| `FIELD_INVENTORY` | `BLOCKED` | 覆盖 descriptor、引用块、prefix、measurement、F01E、optional caches、writer set、receipt owners；每字段标 type、source、consumed/excluded/zeroed 和 profile scope。 | 全字段覆盖审计、未知字段审查。 |
| `CANONICAL_ENCODING` | `BLOCKED` | 定义 schema version、field order、section/length/presence、integer width/endian、F32 canonicalization、null/reserved/padding、pointer 排除、domain separator、digest preimage。 | 独立 schema 重算、跨配置兼容性。 |
| `COLD_VECTOR` | `BLOCKED` | 提供完整 canonical input、expected bytes、length、domain separator 和 digest；不得用诊断 projection 或 expected bytes 作实现控制流。 | 成功 vector、跨 profile rejection vector。 |
| `REFERENCE_ROUNDTRIP` | `BLOCKED` | 洗白侧独立 serializer/parser，只依赖 normative schema；serialize→parse→serialize 必须 byte-exact。 | truncation、invalid enum/NaN、unknown field、cross-profile rejection、成功 round-trip。 |
| `RECEIPT_TRANSACTION` | `BLOCKED` | 定义 typed tokens、receipt fields/digest、prepare/stage/commit/abort/retired/reset 顺序和 durability failure 关系。 | success commit、invalid-input zero-commit、fatal retire、destroy+fresh、partial write vectors。 |

## 结论

v34 的跨 profile actual runs 只覆盖三个指定 route。sidecar parser rejection 和 setter no-hit 不是产品级 absence。selected-run causal join 也不补足 canonical schema、reference serializer、receipt 或 round-trip。全部五个依赖节点未关闭，因此实现和生产门槛仍为 NO。
