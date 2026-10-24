# 模块映射

| 路径 | 职责边界 | 当前实现 |
|---|---|---|
| domain/numeric | 批准数值语义的未来 Functional Core 边界 | 空壳；本轮不实现 |
| domain/state | 共享不可变契约和拒绝错误 | 仅 migration rejection；本轮不扩展 |
| domain/frontend | 选定 XML/DBMD 输入、bed 物理身份和 candidate AXML/CHNA graph/event identity | F01E job/DBMD parser、bed/channel join、candidate real-source physical-host/ATU UID identity、28 单block事件 admission、candidate same-source DBMD parse/join 和 control52 set 已按各自界限验证；SOURCE_BINDING_TO_SELECTED_JOB 未解，未接主 encode；完整 control38、callback state 和 F03/F04 audio wiring 仍 deferred |
| domain/spatial | SourceFlags/CurrentFeatures、spatial_control/spatial_audio boundaries, selected primary installed48 source map/activity, P11 geometry, F04G membership and F04F filter/tail previews | Existing component/LFE/Type10/receiver/mode0 stages now join through `spatial_audio_composer`: g0/g1 stage complete priming roots; g>=2 validates component g、LFE g-2 and reader envelope g/invocation g-2 before one receiver/QMF execution, then returns the canonical F04 after-root and immutable J05 projection. No local commit authority is added. Primary upstream render/matrix/tail joins, all cached drains, short-tail regeneration, application caller and parent transaction remain open. |
| domain/joc/downmix_profile | `JocDownmixProfile` 边界 | 空壳；无常数 |
| domain/joc/huffman | F05QH traversal 边界 | 空壳；无表和遍历实现 |
| domain/joc/emdf | F13 body barrier、ID1 configuration、metadata owner boundary | Same-source ID11/ID2/ID14 proposals and typed ID1 configuration feed a noncommitting `stage_source`; timer and all four queues advance on one clone, source0/1 still execute suppression pops, and source2+ can return EMDF. The leaf returns owned after-state plus specialized receipt and does not retire/install production state. Full meter execution, config[2] typed gating binding, F13C/F12 join, parent install and durable output remain open. |
| domain/joc/synthetic_coordinator (tests only) | F05JL caller-provided receipt validation | `#[cfg(test)]` only; validates six synthetic stage receipts and the final ID14 body; no producer, production API, JOC state delta or root readiness promotion |
| domain/joc/synthetic_coordinator (tests only) | F05JL synthetic receipt validation boundary | `#[cfg(test)]` only; validates six caller-provided synthetic receipts and ID14 body, executes no producer, exposes no production API, creates no JOC state delta and does not promote the `joc` root |
| domain/joc/oamd | F05O rowdata wire/body boundaries, F05T selected trim-section codec, and F05L body/queue boundary | F05O rowdata encoder and single unprotected-body reader plus the F05T stateless trim-body codec/minimal DBMD default projection are locally verified direct slices; whole F05O/F05T semantics and F05L body/queue production remain deferred |
| domain/joc/oamd/reader_target | F05R56 marker/history、Type11 wire roundtrip、reader56 和 callback-local target receipt proposal | selected producer locally verified；不得本地 commit，不包含 F05L queue/body、F05JC/J05 或 parent transaction |
| domain/joc/coefficient_payload、coefficient_fitting、dispatch_state、programme_state | F05JC/F05JL fit、retry、sequence、ID14 proposal 与 selected JOC owner root | J05 projection feeds temporal/geometry/180-cell fit and deterministic retry. Exact `JOCSEL01` canonical bytes bind only true JOC-owned persistent state; cold and nonzero structural vectors pass. Sequence advances only after a complete body and uses the explicit same-callback F04D index. Parent install and real-audio validation remain open. |
| domain/meter | 计量规则和 immutable programme measurement receipt | Narrow meter primitives plus the exact receipt value schema and case-sensitive public gating meanings exist. ID1 nine-field projection passes the approved half-dB/gating cases; the whole meter prepass/finalize producer is not connected and fixture receipts cannot grant READY. |
| domain/transform | 变换领域规则 | 空壳 |
| domain/codec/bitio | bounded MSB bit I/O 边界 | 空壳；无 writer |
| domain/codec/frame_syntax | selected E-AC-3 schema 边界 | 空壳；无字段实现 |
| domain/codec/frame_size | 4,096-byte AU size contract | 空壳；无计算实现 |
| domain/codec/bap | F12A BAP 边界 | 空壳；无算法 |
| domain/codec/mantissa | mantissa production 边界 | 空壳；无算法 |
| domain/codec/frame_builder | validated receipt 消费边界 | 空壳；不计算上游值 |
| domain/codec/core_layout | 5.1 六 carrier 边界 | 空壳；无 sample mapping |
| domain/codec/substream_topology | 1 independent / 0 dependent 边界 | 空壳；无 writer |
| domain/codec/rate_profile | 1,024 kbps / 6 blocks / 1,536 samples / 4,096 bytes 边界 | 空壳；无 rate plan |
| domain/integrity/crc16 | E-AC-3 CRC16 生成和 residual 检查 | 多项式 `0x8005`、初值 0、MSB-first；当前由 F14K keyless root 使用 |
| domain/integrity/keyless_checksum | 公共 keyless profile 校验、CRC32 和 CRC8 | 仅接受 `keyless-crc32-crc8-crc16.v1`；拒绝其他 profile，不接触认证材料 |
| domain/integrity/programme_owner | F14K 单 owner 状态、SOURCE/DISPATCH/AU stages 和局部提交 | keyless root 局部验证；全局 CallbackTransaction、私有认证和 native receipt 格式仍未实现 |
| domain/timeline | 已批准 callback/source/writer/AU 时钟类型边界 | Typed g/q/s/d/writer/j indices and selected conversion rules are implemented and used by J05/F05/F13 boundaries；不得用裸 `usize` 跨域。 |
| domain/publication | 不可变待发布结果与声明 | 空壳 |
| domain/transaction | 纯提交/撤销决策 | 空壳 |
| application | 验证请求并协调各 owner | 仅拒绝 encode |
| io/bw64、pcm | 有界字节、chunk 和 PCM 读取适配 | 空壳；未复制旧扫描器 |
| io/atomic_output | 临时写入、flush、rename 和回滚副作用 | 空壳 |
| CLI | 参数、退出码、未来 I/O 协调 | help/version/拒绝编码 |
| validation/eac3_parser | E-AC-3 structural parser | 空壳；validation only |
| validation/channel_map_validator | channel-map validator | 空壳；validation only |
| validation/topology_validator | substream topology validator | 空壳；validation only |
| validation/frame_report | frame report | 空壳；validation only |
| validation/w04 | W04 objective/render evidence | 空壳；validation only |

以上职责受已批准 GTC、owner projection、profile 表和 104 行规格映射约束。空模块只物化边界，不授权算法实现。当前无未来 profile 抽象、插件或算法 feature 分支。
