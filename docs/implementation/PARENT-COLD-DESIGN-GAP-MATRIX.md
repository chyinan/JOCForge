# Parent cold-state 设计缺口矩阵

## 状态

本文件是实现侧 inventory，不是 canonical schema。它整理 v34 洗白资产和 Wash Agent 的 structured gap response。当前唯一完整 profile 仍是 selected ADM/configuration。Gate 1 必需逆向依赖数为 0。设计和验证未完成前，保持 `IMPLEMENTATION_READY=NO`、`production_allowed=false`。

机器目录见 [PARENT-COLD-FIELD-INVENTORY.json](PARENT-COLD-FIELD-INVENTORY.json)，其哈希清单见 [PARENT-COLD-FIELD-INVENTORY-MANIFEST.json](PARENT-COLD-FIELD-INVENTORY-MANIFEST.json)。独立覆盖检查由 `scripts/check-parent-cold-field-inventory.py` 执行。目录状态仍为 `INVENTORY_INCOMPLETE_NOT_NORMATIVE_SCHEMA`。

## 字段和对象 inventory

| 区域 | 已洗白事实 | 当前分类 | 缺口与关闭条件 |
| --- | --- | --- | --- |
| 120-byte cold descriptor | 布局包含 mode、source count、data-rate enum、指针、mode-specific tuple 和 measurement summary。selected run 是 mode 0、source count 38、data-rate code 5。 | `INVENTORY_PARTIAL` | `+24..+39` 的跨 mode 语义、measurement `+8/+12/+16` 名称和引用块全量语义未构成完整逻辑字段表。需逐字段 schema，注明 consumed/excluded/zeroed。 |
| 44-byte policy block | run109 的 16 对 property/value 已闭合；policy `+2`→property31 mixlevel，`+4`→property36 roomtyp，`+32`→property29 adconvtyp。 | `CURRENT_RUN_VALUES_CLOSED` | 仅限已识别 run/config identity。不得移植到其他 profile。需 schema 定义 logical representation 和跨 profile policy。 |
| 38 个 channel records | 每条记录 6 bytes，字段为 input channel key、host index、source ordinal，各为 U16。 | `FIELD_LAYOUT_CLOSED` | 当前适配器只消费 key；raw records 未交付。需定义各字段在 portable parent state 中的使用和排序合同。 |
| 120-byte packed metadata | selected constructor 写入 106 bytes；14 个 gap 是 `78..87`、`99`、`107`、`118..119`。selected init 将它们置零；`80..87` 的 selected-run 最终值来自 ordinal 3 的 binary64 positive zero。 | `SELECTED_RUN_CAUSAL_JOIN_CLOSED` | 只覆盖当前 ADM/configuration 和 composite generation。generic setter 可替换全部 120 bytes，但不能据此定义所有 write path。需有配置范围及完整 logical field inventory。 |
| Packed metadata provider | selected source 是 MasterSourceImpl。source pointer、composite owner `+264`、producer owner `+88` 和 parent `0x178DC0` 的 pointer/owner/order join 已闭合。 | `CURRENT_RUN_IDENTITY_CLOSED` | 当前 source/master 关系不外推。27×U32 dictionary 在 accepted path 未调用，且没有同 owner/generation join；禁止 offset transplant。 |
| External metadata 其他字段 | 120-byte producer、helper offsets 和 selected gap writes 有 bounded evidence；generic setter whole-copy 有独立路径证据。 | `SCHEMA_INCOMPLETE` | 未知字段、constructor owner/first writer 和独立 dictionary join仍未闭合。需字段 owner、值来源和同代 transfer 证据，或保持字段 excluded 并证明逻辑上不需要。 |
| Measurement | core slots 8/12/28 到 public result 12/16/8 的 binary32×128 映射已闭合；public `+20` 的 measured loudness 是已命名字段。 | `PARTIAL_LAYOUT_CLOSED` | public `+8/+12/+16` 名称未闭合。S15 的 DTHD public Meter 是 vptr+PIMPL 的 16-byte wrapper；S16 的 RTTI 只确认 wrapper family，不能把 PIMPL offsets 当作 DDP public layout。需 named owner/consumer 或明确排除。 |
| Trim payload | 24-byte payload 的 flag、padding、gains、balances、copy 和 consumer permutation 有闭合 evidence。 | `PAYLOAD_LAYOUT_CLOSED` | 不等同于 parent cold-state framing。需决定它如何归入 canonical state、哪些字段被消费，以及 vector 覆盖。 |
| Runtime pointers/PIMPL | selected prefix `+88/+92` 指针在 persistence 前清零；parent runtime allocation 含进程内 pointers 和 nested objects。 | `EXCLUDED_FROM_PORTABLE_BYTES_CANDIDATE` | canonical schema 尚未逐字段确认所有 pointer/opaque/padding 的排除规则。不得序列化地址或把整个 allocation 当作 portable bytes。 |
| Parent host lifecycle | 当前观察到 close/destroy 后为新记录创建 fresh inner object；同对象 restore 未观察到。 | `FRESH_RECONSTRUCTION_OBSERVED` | 这不是 serialization/restore 或 receipt。动态 second run 仍 `NOT_RUN`；需独立验证 token/state retirement 与 fresh reconstruction acceptance。 |
| F01E / optional caches / writer set | framing rules 将 top-level envelope、F01E embedding、optional diffusion/source cache、current flags/current features、writer set 列为缺口或冲突。 | `FRAMING_BLOCKED` | 需每项明确 presence、长度、section、顺序和空/非空编码；禁止 unknown zero-fill。 |

## 逐项来源和字段状态

下表的 inventory key 只用于追踪，不是 canonical field ID。每行的 source key 映射到上表 SHA-256，stage/source 列提供 candidate disposition 的证据依据。`candidate` 列是待审 clean-room 处置，不是规范合同。未观测的值写 `NOT_CAPTURED`，未定的类型或规则写 `NOT_DEFINED`。

### Source register

| Key | Sanitized asset | SHA-256 |
| --- | --- | --- |
| S1 | `SELECTED-CONSTRUCTOR-PREFIX-DYNAMIC-JOIN.json` | `a572639b5263fe5f652dd614ff21bcd3be8bdb4d2e19ec8ab26f69f0ce457cb1` |
| S2 | `PARENT-COLD-CONFIG-SEMANTICS.json` | `551dc12dac533190e72947cef663173042e6dd7a872ab8488f8f8722fa51130f` |
| S3 | `PARENT-PROPERTY-PAIRS-AUTHORITY.json` | `023790a6ee6fe74013407b95be4b89823ce52d4e61f227d06e8b35e4d9c81838` |
| S4 | `CHANNEL-EXTERNAL-METADATA-TRIM-SEMANTICS.json` | `7c9f2f96f2eb5da4e3f2693c5d695bf6d2d184920c2edd62f59392508907c764` |
| S5 | `TRIM-PAYLOAD-PRODUCER-CONSUMER-EVIDENCE.json` | `be74fda055d1b93aefa263750421de68103dfebda4f2266421dbf30bda27d1ae` |
| S6 | `EXTERNAL-METADATA-PRODUCER-COVERAGE-AND-SEPARATE-DICTIONARY.json` | `e5c34227db9f145e1ca71c3ad13ca5c8693f17368c0207d7381a89f25fbaccbc` |
| S7 | `CORE-MEASUREMENT-SLOT-CONSUMER-BOUNDARY.json` | `edc5af39dede9931e41491984bc8c02d1c711c7cdc656e7b970f3173e1f0c484` |
| S8 | `PARENT-METADATA-CAUSAL-JOIN.json` | `c8eb0c15506ac018afde203f549418ef9b30ac1302f8f4fab7e6c4b3d84801a2` |
| S9 | `REQUIRED-FRAMING-RULES.json` | `7396aaf144c102db7e395f090bf645e7db2d827d6bf8ecc3a39f870c73e0373d` |
| S10 | `PARENT-COLD-FRESH-RECONSTRUCTION.json` | `984bf84e2aa3825bd7380b022eb5c264329e53f21dbaa2eb43466ea7ac606ee6` |
| S11 | `PARENT-METADATA-PROVIDER.json` | `3b46aa4a778672a9a9233bd72882305498081f31d45fd5c6aa4e792a52afc9ae` |
| S12 | `MINIMUM-EXTERNAL-EVIDENCE.json` | `7e078cfa1da60dcf0580a5fe070d6255c4c8cfa2265b862242965342210c922f` |
| S13 | `PARENT-OUTPUT-DATAFLOW.json` | `8ec6387277da562610614795459ef03b8c960ea2962a51bb492fcee6fcae3a20` |
| S14 | `CROSS-PROFILE-ACTUAL-RUNS.json` | `13c52fb8b0f0ca0c0f6176165d9993d3a5b5972b56533765098ea09ac4165bcf` |
| S15 | `DTHD-METER-PIMPL-BOUNDARY.json` | `7f33cd81e7a35bdf803efbf3de06b9edacb5c65c4e7711c5457c02bc84032cb5` |
| S16 | `TARGETED-LOUDNESS-TYPE-CROSS-MODULE-NO-HIT.json` | `055cb28aad6e0073873026e60b4ddf3f1a1143b445d0cca62ea8467ae22b337d` |
| S17 | `EXTERNAL-METADATA-STRUCTURE-AND-REMAINING-FIELDS.json` | `3ee2f1a482191ed2352d6d154fcaa6638c82dfdb6f45e510a91413b5f9a85dcc` |

### Selected 104-byte constructor prefix

本表只描述当前 selected run。producer/consumer 由 S1 的同代调用链限定。offset 88、92 是运行期指针，不能编码进 portable state。

| Inventory key | Offset / type | Selected value 或语义 | Stage / source key（basis） | Candidate disposition | Blocker |
| --- | --- | --- | --- | --- | --- |
| `prefix.data_rate` | 0 / U32 enum | 5，1024 kbps | selected constructor → descriptor builder / S1 | candidate consumed | 仅当前 run。 |
| `prefix.line_drc` | 4 / U32 enum | 5，none | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.rf_drc` | 8 / U32 enum | 5，none | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.preferred_downmix` | 12 / U32 enum | 2，Lo/Ro | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.loro_center_mix` | 16 / U32 enum | 4，-3 dB | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.loro_surround_mix` | 20 / U32 enum | 4，-3 dB | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.ltrt_center_mix` | 24 / U32 enum | 4，-3 dB | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.ltrt_surround_mix` | 28 / U32 enum | 4，-3 dB | 同上 / S1 | candidate consumed | 仅当前 run。 |
| `prefix.surround_trim_5_1` | 32 / U32 enum | 5，auto | selected prefix / S1 | candidate unresolved | selected branch consumption `NOT_JOINED`。 |
| `prefix.height_trim_5_1` | 36 / U32 enum | 5，auto | selected prefix / S1 | candidate unresolved | selected branch consumption `NOT_JOINED`。 |
| `prefix.dialogue_loudness` | 40 / F32 | -120.0 | selected prefix / S1 | candidate consumed | field semantics closed only in this scope。 |
| `prefix.itu_loudness` | 44 / F32 | -16.784912109375 | selected prefix / S1 | candidate consumed | field semantics closed only in this scope。 |
| `prefix.measurement_48` | 48 / F32 | -17.434703826904297，name `NOT_CLOSED` | selected prefix / S1 | blocked unknown | 不得按数值或邻近字段命名。 |
| `prefix.measurement_52` | 52 / F32 | -120.0，name `NOT_CLOSED` | selected prefix / S1 | blocked unknown | 不得当作 absent/null。 |
| `prefix.measurement_56` | 56 / F32 | -24.566104888916016，name `NOT_CLOSED` | selected prefix / S1 | blocked unknown | 不得按相同值合并字段。 |
| `prefix.measured_loudness` | 60 / F32 | -16.784912109375 | selected prefix / S1 | candidate consumed | alias为`measurement_result.measured_loudness`。 |
| `prefix.gating` | 64 / U32 enum | 2，level | selected prefix / S1 | candidate consumed | 禁止按 F32 解读。 |
| `prefix.speech_percentage` | 68 / F32 | -1.0 | selected prefix / S1 | candidate consumed | selected run only。 |
| `prefix.loudness_range` | 72 / F32 | 5.73073673248291 | selected prefix / S1 | candidate consumed | selected run only。 |
| `prefix.sample_peak` | 76 / F32 | -4.125058174133301 | selected prefix / S1 | candidate consumed | selected run only。 |
| `prefix.true_peak` | 80 / F32 | -3.954448699951172 | selected prefix / S1 | candidate consumed | selected run only。 |
| `prefix.source_trim_mode` | 84 / U32 discriminator | 0 | selected prefix / S1 | candidate consumed | meaning bounded to observed descriptor source trim mode。 |
| `prefix.priv_pointer_low` | 88 / runtime pointer | zero before persistence | selected prefix / S1 | candidate excluded | address and pointee excluded。 |
| `prefix.priv_pointer_high` | 92 / runtime pointer | zero before persistence | selected prefix / S1 | candidate excluded | address and pointee excluded。 |
| `prefix.backend_selection` | 96 / U32 | 0 | selected prefix / S1 | blocked unknown | scoped inner consumer `NO_HIT`。 |
| `prefix.blu_ray_mode` | 100 / U32 bool | 0 | selected prefix / S1 | blocked unknown | scoped inner consumer `NO_HIT`。 |

### Descriptor、policy 和 measurement fields

| Inventory key | Offset / type | Selected value 或语义 | Stage / source keys（basis） | Candidate disposition | Blocker |
| --- | --- | --- | --- | --- | --- |
| `descriptor.mode` | 0 / U32 | mode 0 | descriptor producer → parent constructor / S2 | candidate consumed | mode 1 tuple仅partial。 |
| `descriptor.reserved_4` | 4 / U32 | zero | descriptor producer / S2 | candidate excluded/zeroed | 需 canonical zero rule。 |
| `descriptor.source_count` | 8 / U32 | 38 | descriptor producer / S2 | candidate consumed | selected run only。 |
| `descriptor.reserved_12` | 12 / U32 | zero | descriptor producer / S2 | candidate excluded/zeroed | 需 canonical zero rule。 |
| `descriptor.channel_records_pointer` | 16 / pointer | 38 records × 6 bytes | descriptor producer / S2 | candidate excluded | pointer不能序列化。 |
| `descriptor.mode_tuple` | 24..39 / 4×U32 | mode 0 zero，mode 1 partial | descriptor producer / S2 | blocked partial | mode 1全字段语义缺失。 |
| `descriptor.data_rate` | 40 / U32 enum | 5，1024 kbps | descriptor producer / S2 | candidate consumed | run109 property table实际为 property3。 |
| `descriptor.reserved_44` | 44 / U32 | zero | descriptor producer / S2 | candidate excluded/zeroed | 需 canonical zero rule。 |
| `descriptor.policy_pointer` | 48 / pointer | 44-byte policy block | descriptor producer / S2 | candidate excluded | policy逻辑字段见下表。 |
| `descriptor.external_metadata_pointer` | 56 / pointer | 120-byte block | descriptor producer / S2 | candidate excluded | metadata semantic inventory不完整。 |
| `descriptor.trim_pointer` | 64 / pointer | 9 × 24-byte payload | descriptor producer / S2 | candidate excluded | portable embedding未定义。 |
| `descriptor.measurement_summary` | 72..115 / 44 bytes | measurement fields below | measurement producer/adapter / S2 | candidate partial | public `+8/+12/+16`仍无名称。 |
| `descriptor.reserved_116` | 116 / U32 | zero | descriptor producer / S2 | candidate excluded/zeroed | 需 canonical zero rule。 |
| `policy.copyright` | policy+0 / U8 | property33=1 | policy producer/property setter / S2,S3 | candidate consumed | selected run scope。 |
| `policy.original` | policy+1 / U8 | property34=1 | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.mixlevel` | policy+2 / U8 | raw0 → property31=80 | 同上 / S2,S3 | candidate consumed | exact transform已观测。 |
| `policy.reserved_3` | policy+3 / U8 | zero | policy producer / S2 | candidate excluded/zeroed | cross-profile规则未闭合。 |
| `policy.roomtyp` | policy+4 / U32 | property36=0 | property converter/setter / S2,S3 | candidate consumed | enum范围按S2。 |
| `policy.loro_center` | policy+8 / U32 | property25=4 | policy producer/property setter / S2,S3 | candidate consumed | selected run scope。 |
| `policy.loro_surround` | policy+12 / U32 | property26=4 | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.preferred_downmix` | policy+16 / U32 | property20=2，Lo/Ro | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.ltrt_center` | policy+20 / U32 | property23=4 | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.ltrt_surround` | policy+24 / U32 | property24=4 | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.dsurexmod` | policy+28 / U32 | property27=0 | 同上 / S2,S3 | candidate consumed | selected run scope。 |
| `policy.adconvtyp` | policy+32 / U8 | property29=0 | converter/property setter / S2,S3 | candidate consumed | selected run scope。 |
| `policy.reserved_33_35` | policy+33..35 / 3 bytes | zero | policy producer / S2 | candidate excluded/zeroed | cross-profile规则未闭合。 |
| `policy.rf_drc` | policy+36 / U32 | property15=0，none | policy producer/property setter / S2,S3 | candidate consumed | selected run scope。 |
| `policy.line_drc` | policy+40 / U32 | property13=0，none | policy producer/property setter / S2,S3 | candidate consumed | selected run scope。 |
| `measurement.dialogue_loudness` | descriptor+72 / F32 | -120.0 | measurement producer/adapter / S2 | candidate consumed | selected run scope。 |
| `measurement.itu_loudness` | descriptor+76 / F32 | -16.784912109375 | 同上 / S2 | candidate consumed | selected run scope。 |
| `measurement.result_8` | descriptor+80 / F32 | observed -120.0，name `NOT_CAPTURED` | measurement export / S2,S7 | blocked unknown | maps to internal/core slot; semantic name missing。 |
| `measurement.result_12` | descriptor+84 / F32 | observed -120.0，name `NOT_CAPTURED` | measurement export / S2,S7 | blocked unknown | maps to internal/core slot; semantic name missing。 |
| `measurement.result_16` | descriptor+88 / F32 | observed -120.0，name `NOT_CAPTURED` | measurement export / S2,S7 | blocked unknown | maps to internal/core slot; semantic name missing。 |
| `measurement.measured_loudness` | descriptor+92 / F32 | result offset+20，-16.784912109375 | meter/adapter / S2, measurement alias asset | candidate consumed | producer/consumer names are aliases of one field。 |
| `measurement.gating` | descriptor+96 / U32 enum | 2，level | measurement export / S2 | candidate consumed | F32 interpretation forbidden。 |
| `measurement.speech_percentage` | descriptor+100 / F32 | -1.0 | measurement export / S2 | candidate consumed | selected run scope。 |
| `measurement.loudness_range` | descriptor+104 / F32 | 5.73073673248291 | measurement export / S2 | candidate consumed | selected run scope。 |
| `measurement.sample_peak` | descriptor+108 / F32 | -4.125058174133301 | measurement export / S2 | candidate consumed | selected run scope。 |
| `measurement.true_peak` | descriptor+112 / F32 | -3.954448699951172 | measurement export / S2 | candidate consumed | selected run scope。 |

### Channel、trim 和 external metadata

| Inventory key | Storage / type | Selected fact | Stage / source | Candidate disposition | Blocker |
| --- | --- | --- | --- | --- | --- |
| `channel_record.input_channel_key` | record+0 / U16 | field name闭合；selected value `NOT_CAPTURED` | resolver/DTO / S4 | candidate consumed | 原始记录未交付。 |
| `channel_record.host_index` | record+2 / U16 | field name闭合；selected value `NOT_CAPTURED` | channel record copy / S4 | candidate unresolved | 当前DTO不消费此字段。 |
| `channel_record.source_ordinal` | record+4 / U16 | field name闭合；selected value `NOT_CAPTURED` | channel record copy / S4 | candidate unresolved | 序号来源分对象/bed；portable order未定。 |
| `trim.flag` | payload+0 / U8 | group flag；selected value `NOT_CAPTURED` | trim adapter/consumer / S5 | candidate consumed | parent framing未定义。 |
| `trim.padding` | payload+1..3 / bytes | unspecified padding；raw bytes `NOT_CAPTURED` | trim adapter / S5 | candidate excluded/zeroed | canonical padding规则需明确。 |
| `trim.gain_0` | payload+4 / F32 | gain0；selected value `NOT_CAPTURED` | trim producer/consumer / S5 | candidate consumed | canonical F32规则未定义。 |
| `trim.gain_1` | payload+8 / F32 | gain1；selected value `NOT_CAPTURED` | trim producer/consumer / S5 | candidate consumed | canonical F32规则未定义。 |
| `trim.gain_2` | payload+12 / F32 | gain2；selected value `NOT_CAPTURED` | trim producer/consumer / S5 | candidate consumed | canonical F32规则未定义。 |
| `trim.balance_0` | payload+16 / F32 | balance0 from signed-magnitude byte；selected value `NOT_CAPTURED` | trim producer/consumer / S5 | candidate consumed | 编码和 rounding规则未定义。 |
| `trim.balance_1` | payload+20 / F32 | balance1 from signed-magnitude byte；selected value `NOT_CAPTURED` | trim producer/consumer / S5 | candidate consumed | 编码和 rounding规则未定义。 |
| `external_metadata.kind` | packed+0 / U8 | `NOT_CAPTURED`，presence/kind partial | producer `0x4E0890` → copy `0x178DC0` / S6 | blocked partial | presence semantics未闭合。 |
| `external_metadata.name` | packed+1..63 / char[63] | truncated UTF-8 field | producer / S6,S17 | candidate partial | business name和portable encoding未闭合。 |
| `external_metadata.name_nul` | packed+64 / U8 | literal zero terminator | producer / S6 | candidate consumed | 与name字段绑定，portable string规则未定义。 |
| `external_metadata.source_copy_65` | packed+65 / U8 | copied source+72，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_66` | packed+66 / U8 | copied source+76，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_67` | packed+67 / U8 | copied source+80，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.monitor_renderer` | packed+68 / converted U8 | helper mapping已闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | mapping不能替代selected input value。 |
| `external_metadata.source_copy_69` | packed+69 / U8 | copied source+88，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_70` | packed+70 / U8 | copied source+92，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_71` | packed+71 / U8 | copied source+96，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.dbmd_master_type` | packed+72 / converted U8 | helper mapping已闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.source_copy_73` | packed+73 / U8 | copied source+104，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_74` | packed+74 / U8 | copied source+108，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.source_copy_75` | packed+75 / U8 | copied source+112，name `NOT_CAPTURED` | producer / S6 | blocked unknown | source field semantic缺失。 |
| `external_metadata.content_type` | packed+76 / converted U8 | helper mapping已闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.frame_rate` | packed+77 / converted U8 | helper mapping已闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.gap_78_87` | packed+78..87 / 10 bytes | selected-init cold zero；`+80..87` same-run final positive zero | constructor/provider/consumer / S6,S8 | scope-limited zero candidate | other generations/profiles not generalized。 |
| `external_metadata.timecode_seconds` | packed+88 / F64 | producer formula闭合，selected value `NOT_CAPTURED` | producer / S6 | candidate consumed | source units和portable encoding仍需定义。 |
| `external_metadata.timecode_present` | packed+96 / bool | producer tests source+124 nonzero，selected value `NOT_CAPTURED` | producer / S6 | candidate consumed | presence contract未定义。 |
| `external_metadata.position_smoothed` | packed+97 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.monitored_bed_configuration` | packed+98 / U8 | source+144 clamp，semantic partial | producer / S6 | blocked partial | semantic mapping未闭合。 |
| `external_metadata.gap_99` | packed+99 / U8 | bounded producer未写 | producer / S6 | scope-limited zero/excluded candidate | owner first-writer需核对。 |
| `external_metadata.room_length_100` | packed+100 / converted U16 | shared helper计算100×low32+high32 | producer/helper / S6 | blocked semantic name | 三个dimension名称未闭合。 |
| `external_metadata.room_length_102` | packed+102 / converted U16 | shared helper计算100×low32+high32 | producer/helper / S6 | blocked semantic name | 三个dimension名称未闭合。 |
| `external_metadata.room_length_104` | packed+104 / converted U16 | shared helper计算100×low32+high32 | producer/helper / S6 | blocked semantic name | 三个dimension名称未闭合。 |
| `external_metadata.spatial_processing_mode` | packed+106 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.gap_107` | packed+107 / U8 | bounded producer未写 | producer / S6 | scope-limited zero/excluded candidate | owner first-writer需核对。 |
| `external_metadata.monitored_coding_mask` | packed+108 / U32 | helper bit remap闭合，invalid default 8 | producer/helper / S6 | candidate partial | transform name/selected input未闭合。 |
| `external_metadata.monitored_coding_code` | packed+112 / U8 | source+180，invalid default 0 | producer / S6 | candidate partial | semantic mapping未闭合。 |
| `external_metadata.downmix_5_to_2` | packed+113 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.phase_shift` | packed+114 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.downmix_7_to_5` | packed+115 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.bed_distribution` | packed+116 / converted U8 | helper mapping闭合，selected value `NOT_CAPTURED` | producer/helper / S6 | candidate consumed | selected input未捕获。 |
| `external_metadata.warm_mode` | packed+117 / converted U8 | helper mapping闭合，prior observed value 4 / S17 | producer/helper / S6,S17 | candidate consumed |仅限 evidence run。 |
| `external_metadata.gap_118_119` | packed+118..119 / 2 bytes | bounded producer未写 | producer / S6 | scope-limited zero/excluded candidate | owner first-writer需核对。 |
| `ddp_dictionary.words[0..26]` | 27 × U32 | program_id、lfeon、bsmod、acmod、cmixlev、surmixlev、dsurmod、langcod_present、copyrightb、origbs、dialnorm_negated、langcod_value、audprodie_present、mixlevel、roomtyp、lorocmixlev、lorosurmixlev、dmixmod、ltrtcmixlev、ltrtsurmixlev、dsurexmod、dheadphonmod、adconvtyp、compr1、dynrng1、stream_type、datarate | separate serializer/parser route / S6,S9 | candidate excluded from packed descriptor | same owner/generation identity join=`NO_HIT_NOT_INVOKED`；不得移植这些名字到 packed120 offsets。 |

### Fresh lifecycle、framing 与 receipt

| Inventory key | Native-observed fact | Clean-room disposition candidate | Blocker / 未运行范围 | Source |
| --- | --- | --- | --- | --- |
| `parent.fresh_inner` | close/destroy 后按记录创建新 EmpireEnc inner。 | 可建模 fresh construction；不得命名为 restore。 | dynamic second run=`NOT_RUN`；receipt retirement vector缺失。 | S10 |
| `parent.metadata_provider` | composite owner+264由 active source whole-copy 120 bytes；v34 selected run已闭合同代 pointer/owner/order join。 | candidate同run consumed。 | cross-profile transfer和portable logical schema未闭合。 | S8,S11 |
| `parent.pointer_state` | runtime pointers/PIMPL存在，+88/+92在 selected prefix persistence前清零。 | candidate excluded。 | 完整 pointer/opaque/reserved inventory仍未进入规范。 | S1,S2 |
| `frame.F01E_embedding` | F01E preimage pinned。 | disposition未选。 | parent embedding/presence/length未定义。 | S9 |
| `frame.optional_caches` | cold cache absent；generic optional presence部分已知。 | disposition未选。 | grouping/order/array length未定义。 | S9 |
| `frame.current_flags_features` | root owners列出，但无独立历史，持久化选择冲突。 | disposition未选。 | 是否persist未解决。 | S9 |
| `frame.writer_set` | cold set empty；记录通用U32 count/length部分已知。 | disposition未选。 | section、record schema、非空编码未定义。 | S9 |
| `receipt.commit` | 没有 native receipt schema。 | washed-only candidate，不得冒称 native。 | owner set、token/generation join、canonical receipt bytes/digest 未定义。 | S10、S12 |
| `receipt.invalid_input` | zero-commit 是验收要求，当前尚无执行 vector。 | candidate transition 待定义。 | abort/保留 staged state 规则与 vector 未运行。 | S12 |
| `receipt.durable_failure` | 文件 append/physical write 证据只说明编码输出。 | disposition 未选。 | partial write、fsync/rename 失败和 receipt 关系未闭合。 | S12、S13 |

### Omitted fields 和完整性结论

本 inventory 只枚举 v34 sanitized evidence 中可定位的字段组。下表列出未能完整展开的项目，不能默认为“无字段”。

| Omitted inventory key | Source/hash | 缺口 | 关闭条件 |
| --- | --- | --- | --- |
| `omitted.mode1_source_tuple` | S2 | descriptor `+24..+39` 的 mode 1 语义 partial。 | mode 1 每个 word 的来源、消费者、类型和值域闭合。 |
| `omitted.metadata_gap_other_profiles` | S6、S8、S14 | 14 个 gap 的其他 generation/profile 写入规则未知。 | 每个目标 profile 独立 owner/write trace；不得从 selected run 外推。 |
| `omitted.measurement_public_names` | S7 | public result `+8/+12/+16` 名称未捕获。 | named owner/consumer 或 debug/PDB binding；RTTI 家族名不够。 |
| `omitted.dictionary_identity_join` | S6、S14 | 27×U32 dictionary 与 packed120 的同 owner/generation join 未建立。 | 同一输入运行捕获 parser/serializer invocation、owner pointer 和 generation。 |
| `omitted.parent_framing` | S9 | F01E embedding、optional caches、current flags/features、writer-set 非空编码未定义。 | 每项明确 presence、order、length、section 和 empty/non-empty 表示。 |
| `omitted.canonical_root_receipt` | S12 | 完整 logical root、digest preimage、receipt fields 和 transition vectors 缺失。 | 先关闭字段 inventory，再按依赖节点定义并独立验证 schema/vector/receipt。 |

因此字段 inventory 自身标记 `INVENTORY_INCOMPLETE`。它可用于阻塞审查，不可作为执行schema或成功cold vector。

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
