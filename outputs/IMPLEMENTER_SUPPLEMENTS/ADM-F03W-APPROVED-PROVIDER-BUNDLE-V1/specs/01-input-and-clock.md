# F01 与 F02 输入及时间

状态：`PARTIAL / SELECTED_FRONTEND_CONFIG_EVENTS_LOCAL_REVIEW_PASS`。选定固定 profile 的 job XML、RIFF/PCM24、ADM 引用图、DBMD、单事件输入、样本计数和尾部行为已定义；任意多事件 ADM 与非零时间配置明确拒绝。其余空间和音频阶段仍见 Q03/Q04。

## F01.1 配置契约

外部请求为 ADM BWF、完整文件时间区间及显式编码配置。`adm` 和 `atmos_mezz` 是当前被接受的输入标签，实际音频按 WAVE/ADM 结构读取。输入来源、输出文件和临时文件路径属于调用者接口；它们不参与数值预测。

profile 字段必须同时符合 `profile.json`。不提供普通编码器或空间配置覆盖。F01E 定义未知字段、重复字段、DTD/entity、无效 XML、namespace、可选字段缺省和所有固定 profile 字段。不能自行把未知配置解释为本 profile。

F01C 是早期前端时钟/layout 子集核验。实现合同以 F01E 为准；F01E 同时检查 loudness、DRC、downmix、trims、输出引用及时间字段，并拒绝重复的已识别单值路径。

F01E 选定配置/事件边界的r3 package-only审查已通过，记录见 [FRONTEND-CONFIG-EVENT-LOCAL-REVIEW-STATUS.json](../test-vectors/FRONTEND-CONFIG-EVENT-LOCAL-REVIEW-STATUS.json)。该记录只令selected frontend config/events local scope为ready；F01 umbrella仍为PARTIAL，不覆盖完整音频、任意ADM时间线或整包验收。

当前 codec 有 5 个全带宽通道、一个 LFE、6 块/帧，acmod=7、sample_rate=48000、码率 1024 kbps。metadata 预算选择的 ID 为 1 和 6，当前不扣预算，带宽码 48，全带宽 bin 数 `3×48+73=217`。这组结果是配置规则的当前分支，不能用它推断其他 profile。

## F01.2 RIFF 边界和 chunk 读取

输入为完整 bytes。至少12 bytes，前四字节为 ASCII RIFF，第8～11字节为 WAVE。第4～7字节的 U32LE 加8必须等于实际文件长度。当前分支拒绝 RF64、BW64、WAVE_FORMAT_EXTENSIBLE；不把这些输入自动转成当前 profile。

从位置12开始，尚有至少8 bytes时读取4-byte chunk ID 和 U32LE payload 长度。payload 的结尾必须在 RIFF 范围内。下一 chunk 位置为 payload_end+(length mod2)；奇数长度的 padding 不属于 payload，padding 的内容不参与 PCM。未知 chunk 跳过；输出 chunk ID 序列保持原顺序。ID 按 ASCII 解码，无效 byte 使用 replacement character。

fmt 必须恰好一个且 payload 至少16 bytes；按 U16LE format、U16LE channels、U32LE sample_rate、U32LE byte_rate、U16LE block_align、U16LE bits_per_sample 读取前16 bytes，其后数据不消费。data 必须恰好一个、非空。当前参数为 format=1、channels=38、sample_rate=48000、bits_per_sample=24、block_align=114、byte_rate=5472000；data 长度必须被114整除。

读取器保留 axml、chna、dbmd、bext、iXML 的原始 payload。重复 metadata chunk 在容器扫描时保留，但请求某种 metadata payload 时若该 ID 出现多次则失败。has_adm 表示 axml 和 chna 均存在，仅为存在性检查；不能据此认为 XML/link 语义已通过。

接受模型的扫描没有要求最终游标等于 RIFF 结尾：最后不足8 bytes的尾部不扫描；最后奇数 payload 的 padding 可以越过结尾一字节。合成边界向量保留这一行为。原生程序对这些非规范输入的行为尚未核验；当前歌曲不触发这些分支。不得把模型读取器的边界行为宣称为通用 RIFF 或原生容错规则。

输出为 sample_rate、channels、block_align、data_offset、data bytes、chunk ID 序列和 metadata payload 序列。源样本数 N=data_length/114。该读取阶段不建立轨道到 bed/object 的映射；映射由 F01G、F01E 和 F01GL 使用 `channel_format_id` 与物理 slot 建立。

## F01.3 PCM24 转换及 hop

输入 sample_index 和 track_index 是整数，bool 不可充当整数；范围分别为0～N-1和0～37。样本 byte 起点为 data_offset+114×sample_index+3×track_index。以 b0+256b1+65536b2 读取 U24LE；bit23置位时减去2^24，得到 signed PCM24。

输出为 R32(value/2^23)，以小端4-byte F32表示。所有 PCM24 整数除以2^23均可精确表示为 binary32；最大正值小于1，最小负值为-1。PCM零输出 positive zero，不使用额外 gain、dither 或归一化。

hop 的 first_sample 为非负整数，sample_count 为正整数，first_sample+sample_count<=N。返回38个 planar byte rows，按物理轨道顺序，每行 sample_count×4 bytes。只转换请求的有效样本，不在 PCM adapter 内补齐1536；padding 在 F02.1 执行。输入范围、格式或 chunk 校验失败时不产生可用 hop，并按 I03 终止 programme。

## F02.1 完整 host frame

正常读取返回 `r` 个有效源样本，`1<=r<=1536`。38 路目标缓冲先清零；每路把这次 `r` 个 binary32 样本写到首部，剩余 `1536-r` 个样本保持 positive zero。host 描述符的样本数始终为 1536。PCM24 转换依 F01.3；ADM chunk/link 和事件语义依 F01G/F01E。

## F02.2 持续时间公式

令源样本数为正整数 `N`，`A=ceil(N/1536)`，`P=1536A-N`。当前 profile 的前端延迟和后端调度延迟各为 3072 样本。flush 数为 `(3072+3072)/1536=4`。

总前端 callback 数 `A+4`；有效六路帧数 `A+4-2=A+2`；编码器吞入相同数量的 source frame。再抑制 2 个 codec source frame，发布数量为 `A`。该计数公式已与现有模型核验；其他输入长度的完整音频结果仍须独立验证。

测量通路的延迟为 3104 样本，需要 `ceil(3104/1536)=3` 个尾部 callback，callback 数 `A+3`。测量有效样本裁剪与窗口边界属于 F06，不能把 3 次与主通路的 4 次混用。

## F02.3 EOF 状态机

`READING`：读取、补正零、处理完整 host frame。保存最后一次正常 callback 的完整 padded 输入。

`CACHED_FLUSH`：正常输入 EOF 后，对保存的同一组 38×1536 样本依次处理 4 次。每次推进前端状态及 callback 时钟，剩余次数减一；不新增源样本，不重新零填整个输入。

`DONE`：flush 次数为零后关闭前端输出。普通 codec 的 flush 模式为 none；只编码已经产生的有效六路 source frame。

空输入拒绝。重置应丢弃上一个 programme 的 EOF 缓存、计数和全部跨 callback 状态；前端的完整冷初始化清单属于 Q04，当前仍缺条款。

## F02.4 窗口

3/1、39/37、94/92、4249/4247 表示 codec source/published 数量。前三者均是完整歌曲冷启动流的前缀，不是将 WAV 截断后重新执行 EOF。对应主前端处理到 5、41、96 次 callback 和整首 4251 次 callback。

各窗口重新冷启动，不从前一窗口缓存继续作为唯一验收。另做一次连续扩展运行，对前缀输出一致性进行检查。
