# 模块映射

| 路径 | 职责边界 | 当前实现 |
|---|---|---|
| domain/numeric | 批准数值语义的未来 Functional Core 边界 | 空壳；本轮不实现 |
| domain/state | 共享不可变契约和拒绝错误 | 仅 migration rejection；本轮不扩展 |
| domain/frontend | 输入语义验证与规范化 | 空壳 |
| domain/spatial | SourceFlags/CurrentFeatures、spatial_control 和 spatial_audio owner boundaries | SourceFlags/CurrentFeatures、spatial_control cold/staging、F04 providers、固定 64 点 QMF FFT primitive、E2 component stage、E3 Direct-LFE ring stage、E4 Type10 assembly boundary、typed PostWindow fanout、固定 QMF analysis/synthesis core 和 receiver mode=1 routing/state stage 已按限定切片验证；F04R 到 s09_input 的生产、整条 F04 receiver、mode=0 fitting、J05 投影执行和 parent commit 仍 deferred |
| domain/joc/downmix_profile | `JocDownmixProfile` 边界 | 空壳；无常数 |
| domain/joc/huffman | F05QH traversal 边界 | 空壳；无表和遍历实现 |
| domain/joc/emdf | F13 EMDF production 边界 | 空壳；无 serializer |
| domain/joc/oamd | F05O shared wire and F05L body/queue boundary | F05O shared wire and F05L body/queue production remain deferred; selected reader feedback is isolated in `reader_target` |
| domain/joc/oamd/reader_target | F05R56 marker/history、Type11 wire roundtrip、reader56 和 callback-local target receipt proposal | selected producer locally verified；不得本地 commit，不包含 F05L queue/body、F05JC/J05 或 parent transaction |
| domain/joc/coefficient_payload | F05Q/F05JL coefficient body 边界 | 空壳；无量化或 payload |
| domain/meter | 计量规则 | 空壳 |
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
| domain/integrity/crc16 | 公共 CRC16 production 边界 | 空壳；无多项式或实现 |
| domain/integrity/keyless_checksum | F14K keyless receipt 边界 | 空壳；无 checksum 实现 |
| domain/timeline | 已批准 callback/source/writer/AU 时钟类型边界 | 空壳；关系见 CLOCK-MAP |
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
