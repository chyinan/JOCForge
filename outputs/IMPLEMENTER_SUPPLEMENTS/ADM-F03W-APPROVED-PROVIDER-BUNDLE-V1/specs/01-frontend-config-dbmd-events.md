# F01E 固定前端配置、DBMD 和 ADM 事件生产

状态：`LOCAL_REVIEW_PASS / WHOLE_ACCEPTANCE_PENDING`。

本条款定义选定输入域的完整配置和事件生产。输入域固定为 48 kHz、38 路 PCM24、10 路床和 28 路对象。码率固定为 1,024 kbps。每个对象只有一个零起点 Cartesian block。该 block 覆盖完整源节目。其他 ADM 时间线必须拒绝。

本条款只含普通功能常数。它不需要私有 key、认证材料、DLL、原表或源码。

## F01E.1 Job XML 入口

输入为 UTF-8 XML 文本、已验证的 WAVE 描述符和源样本数 `N`。WAVE 描述符必须为 `sample_rate=48000`、`channels=38` 和 `pcm_bits=24`。`N` 必须为正整数，`bool` 不能充当整数。

XML 不得含 `DOCTYPE` 或 `ENTITY`，匹配时不区分 ASCII 大小写。XML 必须可解析。根元素必须是无 namespace 的 `job_config`。下述路径均指无 namespace 的直接子路径。每个已识别的单值路径最多出现一次。重复路径拒绝。未列出的元素不参与本 profile，但实现不得让它们覆盖已识别字段。

输入先查找 `input/audio/adm`，不存在时再查找 `input/audio/atmos_mezz`。两者同时存在时选 `adm`。所选节点必须提供 `file_name` 和 `storage/local/path`。这两个 text 不 trim，只作为调用者的 I/O 引用。配置生产器不得打开或修改对应路径。

所选输入的 `timecode_frame_rate` 必须为 `not_indicated`。`offset` 和 `ffoa` 缺失、无 text 或 text 恰好为空时，缺省为 `auto`。其他值先 trim，再要求等于 `auto`。仅含空白的 text trim 后为空，因此拒绝。

编码器节点为 `filter/audio/encode_to_atmos_ddp`。它必须提供以下值：

| 路径 | 接受值 |
|---|---|
| `data_rate` | 十进制 `1024` |
| `timecode_frame_rate` | `not_indicated` |
| `loudness/measure_only/metering_mode` | `1770-4` |
| `loudness/measure_only/dialogue_intelligence` | `false` |
| `loudness/measure_only/speech_threshold` | `100` |
| `drc/line_mode_drc_profile` | `none` |
| `drc/rf_mode_drc_profile` | `none` |
| `downmix/preferred_downmix_mode` | `loro` |
| 四个 center/surround mix level | `-3` |
| `custom_trims/surround_trim_5_1` | `auto` |
| `custom_trims/height_trim_5_1` | `auto` |

四个 mix level 是 `loro_center_mix_level`、`loro_surround_mix_level`、`ltrt_center_mix_level` 和 `ltrt_surround_mix_level`。

`start` 缺失、无 text 或 trim 后为空时取空串，也可显式为 `first_frame_of_action`。`end` 同理，也可显式为 `end_of_file`。`time_base` 必须为 `file_position`。`prepend_silence_duration` 必须为 `0` 或 `0.0`。`append_silence_duration` 缺失或无 text 时缺省为 `0.0`，显式值只能为 `0` 或 `0.0`。空白 text 不等于缺失，必须拒绝。

输出必须有一个 `output/ec3` 节点，并提供非空 `file_name` 和 `storage/local/path`。配置生产器只返回引用，不执行写文件。

## F01E.2 归一化配置

成功时输出以下固定值：

| 字段 | 值 |
|---|---:|
| `bit_rate_kbps` | 1024 |
| `bit_rate_enum` | 5 |
| `layout_enum` | 8 |
| `target_capacity` | 15 |
| `sample_length_enum` | 1 |
| `frame_samples` | 1536 |
| `spatial_config_mode` | 0 |
| `dispatch_mode` | 2 |
| `route_virtual_prefix_count` | 11 |

同时输出 F01E.1 的 loudness、DRC、downmix、trim 和时间字段。下游必须消费这份归一化结果。不得从文件名、旧运行输出或未定义的构造器字节补配置。

## F01E.3 DBMD section framing

输入为 RIFF `dbmd` chunk 的 payload。最少 4 bytes。首个 U32LE 若等于 `chunk_length-4`，则拒绝。解析从 offset 4 开始。

每个非零 section 依次为 `id:U8`、`length:U16LE`、`body[length]` 和 `checksum:U8`。必须满足：

```text
checksum = -(length + sum(body bytes)) mod 256
```

section ID 不得重复。ID 0 是终止符。终止符及其后所有 byte 必须为 0。终止符可省略，但最后一个 checksum 后必须恰好到达 chunk 结尾。截断 header、body 或 checksum 均拒绝。

当前 profile 必须有 section 9 和 section 10。section 9 body 恰好 248 bytes。section 10 body 至少 142 bytes，且 offset 0 的 U32LE 为 `0xf8726fbd`。

## F01E.4 authored mode 与 surround EX

令 `warm_trim_mode = section9[152] & 7`。只接受 0～3。再计算：

```text
authored_source_mode_byte = warm_trim_mode + 1
surround_ex_mode = 2 if authored_source_mode_byte == 3 else 0
```

`warm_trim_mode` 与 `authored_source_mode_byte` 是两个域。当前真实输入的 `warm_trim_mode=3`，因此 host byte 为 4，`surround_ex_mode=0`。不能把 trim mode 3 直接代入 surround EX 判断。

## F01E.5 DBMD per-track 字段

section 10 的 offset 4 为 `track_count:U16LE`。要求 `track_count<=2048`，body 长度至少为 `142+2×track_count`。

第一组数组从 offset 142 开始，共 `track_count` bytes。第二组紧随其后，也有 `track_count` bytes。对物理轨道 `i`：

```text
raw_flag = first[i] & 3
packed = second[i]
code48 = (packed >> 6) & 3
code52 = packed & 7
enum48 = code48 if code48 in {0,1} else -1
enum52 = code52 if code52 in {0,1,2} else -1
flag64 = 1 if raw_flag == 1 else 0
```

这三个字段按物理 `host_channel_index` 连接到同一 ADM 轨道。不得按 audioObject ID、pack ordinal 或排序后对象 ordinal 连接。当前对象 `control52` 不消费这三个字段，但事件记录必须保留它们，供其他 metadata 消费者使用。

## F01E.6 Bed pack 与物理轨道连接

输入为 DirectSpeakers master pack 的 10 个直接 `audioChannelFormatIDRef`，以及 CHNA 引用图解析得到的 10 条物理 bed 轨道。每条物理轨道必须含 `channel_format_id`、`host_channel_index`、`semantic_role` 和 `bed_id`。

pack 引用和物理 `channel_format_id` 必须各自唯一，且两个集合完全相同。对每个 pack 引用，按完全相等的 `channel_format_id` 查找物理轨道。输出保留 pack ordinal 和实际 `host_channel_index`。不得用 pack ordinal 代替物理轨号，也不得只验证集合后按两个数组的位置配对。

合成重排验收把 pack 顺序改为 `D,A,J,B,I,C,H,E,G,F`，物理顺序保持 `A..J`。输出物理 slots 必须为 `3,0,9,1,8,2,7,4,6,5`。

10 条 bed 物理轨道的 `host_channel_index` 必须互异且位于 0～37。28 条对象轨道占用其补集。两组必须无交集，合并后完整覆盖 0～37。床和对象可以任意交织。

床控制行按物理轨号安装。`bed_id=3` 的唯一 LFE 从 regular 音频中删除。其余 37 路按物理控制行递增顺序紧密映射。pack ordinal 只用于 `bed_input_id`，不能改变物理控制行或 compact source 次序。

## F01E.7 对象事件接受域

每个对象轨道必须满足：

- 物理 `host_channel_index` 唯一、位于 0～37，并按递增顺序提交。28 个对象 slots 必须恰好等于 10 个 bed slots 在 0～37 中的补集。
- 恰好一个 Objects block。
- object start 和 block rtime 均转换为 sample 0。
- jumpPosition 为 1，interpolationLength 转换为 0 sample。
- object duration 与 block duration 转换到相同正样本数，且不小于 `N`。
- Cartesian 为 1。X、Y、Z 必须有限。缺失坐标分别取 0.5、0.5 和 0 的 normalized 值。
- width、depth 和 height 若存在，值必须相同。缺失时 width 为正零。diffuse 缺失时为 0。

当前 profile 对多 block、多事件、非零起点、事件间空洞、事件重叠和 programme 中途结束统一拒绝。实现不需要猜测事件优先级。ADM 字段与 F01E.5 的 DBMD 字段先按同一物理轨道合并，再生成控制行。

## F01E.8 坐标、sampler 与 spread

以下 `R32` 表示每步舍入到 IEEE binary32，roundTiesToEven。`R64` 为 binary64。运算不得融合。

存在的 ADM X、Y、Z 先在 R64 检查有限性。恰好等于 −1000 时只跳过输入域夹取；其他值夹到 `[−1,1]`。随后仍执行轴变换：

```text
x0 = R32(R64(R64(X * 0.5) + 0.5))
y0 = R32(R64(0.5 - R64(Y * 0.5)))
z0 = R32(Z)
```

这一步产生 event 坐标。随后 event→record writer 计算 `xy_sentinel=(x0==−1000 且 y0==−1000)`。raw X/Y 同时为 −1000 时，轴变换已经得到 `x0=−499.5`、`y0=500.5`，所以 `xy_sentinel=false`。writer 分别把 x0、y0 夹到 `[0,1]`，得到 0 和 1；z0 除恰好为 −1000 时保留外夹到 `[−1,1]`。sampler 使用 writer 夹取后的 XYZ。

因此合成输入 `X=Y=−1000,Z=−1` 的最终 control52 XYZ 位模式必须是 `00000000,3f800000,bf800000`。不得把输入 sentinel 直接复制进 control52，也不得跳过轴变换。

sampler 固定为 48 kHz、period 512 和 3 slots。常数 `tau` 的 F32 word 为 `0x3ccccccd`。权重由下式生成，不读取表：

```text
denominator = R32(R32(48000) * tau)
ratio = R32(R32(512) / denominator)
history = R32(exp(-ratio))
incoming = R32(R32(1) - history)
```

首次有效事件先把 `previous=(x0,y0,z0)`。随后按 slot 1、slot 2 顺序各执行一次：

```text
previous[k] = R32(R32(xyz[k] * incoming) + R32(previous[k] * history))
```

输出 XYZ 为第二次更新后的 `previous`。即使输入在数学上恒定，也必须执行两次 R32 更新；部分坐标会改变 1 ULP。

width 等于 −1000 时保留，否则夹到 `[0,1]` 后 R32。三个 spread 分量都取该 width。令 `third` 的 F32 word 为 `0x3eaaaaab`：

```text
uniform_width = R32(R32(R32(width + width) + width) * third)
```

不能直接复制 width。`0.1f` 经该式得到 F32 word `0x3dccccce`。

## F01E.9 对象 control52

输出为 13 个四字节槽。槽 5、6、7、9 是 F32LE，其余槽为 U32LE。完整选定对象分支如下：

| 槽 | 类型 | 生产值 |
|---:|---|---|
| 0 | U32 | 0，表示 enabled |
| 1 | 未消费 | 0，隔离接口自有值 |
| 2 | U32 | 3 |
| 3 | U32 | 2 |
| 4 | 未消费 | 0，隔离接口自有值 |
| 5～7 | F32 | F01E.8 的 sampler XYZ |
| 8 | U32 | 0 |
| 9 | F32 | F01E.8 的 `uniform_width` |
| 10 | U32 | `int(diffuse != 0)` |
| 11 | U32 | 0，当前 block component 分支 |
| 12 | U32 | 1，Z 字段启用 |

regular 音频消费槽 0 和槽 9。F06OM 消费槽 0、2、3、5～12，但不消费槽 1 和槽 4。DBMD 的 `enum48`、`enum52` 和 `flag64` 不得塞入槽 11 或槽 12。

## F01E.10 callback、hold、EOF 和 reset

selected cold与state serialization的权威provider为 `parameters/f01e-selected-constructor-v1.json`。owned event state是28条object nested records（ordinal、runtime host、optional event、sampler presence与XYZ），而不是37×13 control数组。cold的physical binding为null；解析后按`channel_format_id`、semantic role和object graph生成10个bed hosts、28个object hosts，要求两组互斥且并集恰为0..37。唯一semantic LFE的host可位于任意物理槽；`regular37_host_order=sort({0..37}−{lfe_host})`。`control38[38][13]`和`regular37[37][13]`都只在完整preflight后按该binding派生，成功时才可整体保存为optional snapshot。F03W的event_control必须直接采用同一nested schema，不得建立第二份37-row sampler owner，也不得硬编码默认歌曲的host排列。

令 `A=ceil(N/1536)`。normal callback `g` 的源起点为 `1536×g`，其中 `0<=g<A`。每次 normal callback 采样同一个零起点事件。事件覆盖完整节目，因此 28 条对象控制保持有效。

最后一次 normal callback 的有效样本数为 `N-1536×(A-1)`。源 PCM 的尾部补零不改变对象控制。normal EOF 后执行 4 次 cached drain。每次 drain 重用最后一次 normal callback 的完整控制快照，不推进 ADM 事件时间，也不重新应用新事件。

当前真实输入 `N=6,522,906`，因此 `A=4,247`。最后一次 normal callback 从样本 `6,521,856` 开始，含 1,050 个有效样本。随后有 4 次 cached drain。

reset 必须丢弃事件 cache、sampler previous、控制快照、callback 计数和 DBMD 绑定。新 programme 从冷态重新解析。任何配置、DBMD、引用图或事件失败都不返回部分控制，不允许沿用上一个 programme 的状态。

## F01E.11 验收范围

主输入向量覆盖 11 个配置案例、4 个 authored mode、6 个 per-track packed 字段、4 个 bed 身份连接、3 个 control52 数值案例、1 个 callback 时钟、1 个 reset 和 6 个事件拒绝，共 36 项。另有 10 个 DBMD malformed/fail-stop 案例，覆盖 checksum、重复 section、截断、terminator tail、warm mode、magic 和 track table。r3 修正 X/Y sentinel 的 event→record 夹取，并保留完整 preimage。

向量内嵌 11 份完整 XML、6 份完整事件拒绝 preimage、clock 的 28 个对象输入与 4 份 drain 期望、reset 前后各 28 个对象输入，以及所有失败前后状态。真实输入只读取 RIFF metadata chunk，跳过 `data` payload。28 条对象的输入和 364 个期望 control52 词全部显式交付。4,249 个 codec source 均逐项给出 warm mode、host byte 和期望 surround EX，不以 digest 替代字段。

本条款闭合选定 profile 的 Q01 配置、DBMD authored mode、单事件合并、hold 和 bed 身份连接。它不声称支持任意 ADM 多事件时间线。

r3 package-only fresh review 已完成。独立 checker 重算36个主案例、10个DBMD malformed/fail-stop案例、11份完整XML、6份事件拒绝preimage的169行、28个实际对象的364个control52词、4,249行BSI、1,456个drain词和1,092个reset词，结果全部匹配。raw sentinel `-1000/-1000` 经 event `-499.5/500.5`、writer clamp `0/1` 到最终control52 `00000000/3f800000/bf800000` 的无bypass顺序也已独立通过。局部状态见 [FRONTEND-CONFIG-EVENT-LOCAL-REVIEW-STATUS.json](../test-vectors/FRONTEND-CONFIG-EVENT-LOCAL-REVIEW-STATUS.json)。该结论不包含任意ADM多事件、完整前端音频、空间质量或整包就绪。
