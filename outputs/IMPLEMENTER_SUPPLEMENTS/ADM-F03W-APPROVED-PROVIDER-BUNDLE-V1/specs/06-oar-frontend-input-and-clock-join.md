# F06OF：meter frontend 到 OAR 的输入和时钟连接

## F06OF.1 选定域和批次

本接口连接 meter frontend 与 F06OC。固定 48000 Hz、H=1536、38 路外部 PCM24、15 个 component rows 和 1 路 direct LFE。meter 创建自己的冷实例，不取得主编码器六声道 PCM 或状态。这里定义已执行模型路线，不要求复制原生 queue、snapshot 或内存 ABI。

frontend callback 用零起始整数 g。F06OC packet 的 generation=g+1，phase 为 NORMAL 或 DRAIN。一个 callback 完成后，才同时交付 SCP16 和 control；不能拼接两个 callback 的输入。SCP 为 16×1536 个有限 F32。row0 是本次 allocated delayed LFE，row1～15 按 component_rows 顺序排列。F32LE 文件按 row、sample 排列，共 98304 字节。

同代表示同一次 producer 调用交付，不表示所有 metadata 来自 g。regular metadata 的 producer_generation 随记录复制。LFE 延迟两帧；component 的历史另有时间域。字节相等不能决定 provenance。

F06OC controls 为 16×13 个 U32。regular 行是本次 delivered control52 的 13 个小端词。direct 行规范为 [0,0,3,0,0,0,0,0,0,0,0,0,0]。F06OM 的 direct 分支只消费 word2=3。其余词是独立载体的未消费零词，不是原始未知内存值。F06OC 继续检查 controls、PCM 和 profile 的合法性。

## F06OF.2 正常读取、末帧和排空

source_samples=N 是整数 1～2^63−1。正常帧数 Q=(N+1535)//1536，总 callback 数 Q+3。源数据大小须为 38×3 的整数倍，并与 N 一致。每路 PCM24 解码仍由其输入模块规定，本接口不重新定义归一化。

g<Q 时，读取每路 [gH,min((g+1)H,N))。actual=min(H,N−gH)，源游标从 min(gH,N) 变为 min((g+1)H,N)。host_input_count=actual；每路末尾补 H−actual 个正零 F32。frontend_samples 始终为 H。补零只用于原始输入，不能在 SCP 输出上再清尾。

g≥Q 时 phase=DRAIN。共三次 DRAIN，其数量为 ceil((3072+32)/1536)。不再执行源读取，源游标保持 N。host_input_count=0、frontend_samples=H；原始输入为新建 38×1536 个正零。三个排空 callback 都完整推进 frontend 状态。

每次的推进顺序为：render 和 audio_state；source flags 与 flag_history；feature FFT/window；prepare、候选 selector/U04、finish，包括 assignment、outer、marker 和 metadata_history；component 和 overlap/filter/history；direct LFE ring；最后组合本次 SCP/control。候选缺席的分支由上游算法定义，不凭本连接猜测。

原始排空输入为零，不推出 SCP 为零，也不推出 control 等于最后正常帧。禁止重复最后 SCP 或 control 代替推进。这里没有额外 crop 或第二次丢弃 3072 样本；shared DV32 与发布时钟交给 F06D/F06W。

选定整曲 N=6522906：Q=4247，总数 4250。最后正常帧从 6521856 开始，1050 个真实样本加 486 个正零。三个 DRAIN 的 packet generations 为 4248、4249、4250。

## F06OF.3 direct LFE ring

ring 有三槽，每槽为尚未写入的 null 或一个完整 1536 F32 字节帧，同时持有输入 generation。冷状态 write_index=0、read_index=1、samples_consumed=0，三个帧和 generation 都为 null。

advance(g,input_frame) 要求 g=samples_consumed//1536，帧长 6144 字节。先将 write_index 和 read_index 各加 1 后模 3，再把本次 input_frame 和 g 写入新 write 槽。根据推进前的 samples_consumed 判断输出：小于 3072 时无有效输出；否则取得新 read 槽的完整帧和 provenance。最后 samples_consumed 增加 1536。

前两次无有效输出时，组合器为 allocated LFE row 明确新建 1536 个正零。它不读取未写槽，也不声称槽冷内存本来全零。从 g=2 起输出来自输入 g−2。首两个 DRAIN 输出最后两个正常 LFE 输入；第三个输出第一个 DRAIN 的零输入。

input_frame 是上游按已定义路由生成的 current LFE 行。输入路由、PCM24 解码及求和数值仍由上游模块定义。本接口不以外部 WAV ordinal 代替逻辑源映射。

## F06OF.4 regular metadata history

history 有三个槽，每槽恰为 15 个独立 record56。record56 是 14 个小端 U32。冷构造词为 [0,1,0,3,2,0,0,0,0,0,0,0,0,1]。每个冷记录的 producer_generation=null，producer_row 为其 row；null 表示冷构造身份，不是输入 generation 0。初始 15 个 marker 为 0。

每次输入恰为 15 个完整 record56、新记录的 producer_generation=g，以及 15 个 I32 marker。对每 row 独立执行：先 slot0←旧 slot1；marker=−1 时新记录写入 slot0/1/2，marker=0 时写入 slot1/2，marker=1 时只写 slot2，其余 I32 不插入新记录。此调用不执行 slot1←旧 slot2。

插入完成后，若 slot0.word1 和 slot1.word1 均非零，则 slot1←slot0。每次复制同时保留全部 56 字节、producer_generation 和 producer_row。最后交付 slot0 的字节 [4,56)，即 control52；provenance 另存，不塞入 F06OC 的四键 packet。

本条定义记录的保存和交付，不生成 marker 或 fit record。marker、selector/U04、component 与配置初始化的算法仍须上游完整提供。验收中的任意 I32/U32 历史输入只检验复制接口，不宣称这些记录可通过 F06OP 的 point profile。

## F06OF.5 direct defined range 与 owned 适配

前两个 callback 的实际 direct control52 已定义全部 13 词：[1,0,3,2,1,3,0,0,0,0,0,0,1]。之后只确定前六词 [0,0,3,2,1,3]，偏移 24～51 的 28 字节未定义。本连接不请求、保存或交付这些尾字。

实际 consumer 仅消费 word2。因此 .1 的 owned 适配为每个 callback 固定填 direct word2=3，其余未消费词为 0；15 个 regular controls 完整复制。规范词不保留真实 direct control 中其他未消费的已知前缀，也不依赖未知尾部。若将来新增 direct 字段消费，须补齐该字段 producer，本规则不继续豁免。

适配不改变 PCM 位模式，尤其不把负零变成正零。所有 regular 控制词均原样小端解码，之后由 F06OC 执行对象 float 重解释、有限性和 profile 检查。

## F06OF.6 冷构造、reset 和失败

整体冷启动分别构造 render/audio_state、feature runtime、front prediction、component/filter parameter 状态及 ring/history。已连接的冷字段包括：front callback_index 和累计样本为 0；previous marker 为 15 个 0；previous XYZ 为 45 个 0；previous IDs 为 15 个 0xffffffff；每个 filter_history 为 256 个零字节，共 15 个；每源 flag_history 为 6144 个零字节。配置派生 attributes 和其余状态的数值由对应上游规格定义。

reset 必须销毁当前完整 OWN pipeline，再按每个模块冷规则重建，generation 回到 0，F06OC callback_index 回到 0。不能只清 OAR gains 而保留旧 frontend history。未定义一个原地 SDK Reset ABI，也不将“调用原 Python 构造函数”交给实现者作为规则。

输入 type、shape、整数范围或长度非法标为 OF_INPUT_ERROR；generation 不连续、phase 与 Q 不符或结束后调用标为 OF_CALLBACK_ORDER；有效输出意外指向 null ring 槽标为 OF_STATE_ERROR。整数不接受 boolean。参数或上游 profile 未供应时按相应模块错误拒绝，不生成正常 SCP。适配输入为 15 个恰长 52 字节的 regular controls，数量或长度错误先拒绝，不返回部分 controls。

这些是隔离接口错误标签。错误发生后按 I03 停止并退役当前 pipeline，整体 reset/rebuild 才可恢复。输入和输出必须独立拥有可变数组；记录 payload 的不可变字节可以共享。成功只提交一次同代批次，不能让新 callback 覆写正在消费的数组。

## F06OF.7 验收与未解决范围

`test-vectors/OAR-FRONTEND-CLOCK-JOIN-VERIFICATION.json` 提供 input-only 调度、七步 LFE ring、连续 metadata history 和 owned control 适配用例。完整下一状态和输入均可独立重算；不包含产品源码或原始研究表。

四个 cold marker 用例是单行投影，检验 .4 对每 row 独立执行的规则，不新增 capacity=1 的产品配置。连续 history 用例使用完整 15 行，覆盖不同 marker、quiet-copy，以及字节相同而生成代不同的记录。调度 digest 按 UTF-8 JSON 的 sorted keys、逗号和冒号分隔、无空格序列化后计算 SHA256。

另有 4250×4=17000 个实际 OWN stored consumer/provenance 文件的既有证据审计。审计逐个核对长度、SHA256 和写边界 ledger，核对 callback 游标、padding、row 配对及 direct 已定义范围。这是既有文件连接审核，没有重新从 ADM 执行完整冷 frontend。

第三个 DRAIN 的一些 SCP rows 有 1535 个非正零 U32 字。这一计数包含负零，不能据此宣称有非零数学幅度或可听尾声。无论正负零或真实尾部，consumer 都必须接收实际 SCP。

本模块只闭合时钟、状态复制和批次连接。render、feature、selector/U04、outer、marker、component 和其必要音频参数仍需逐模块供应、审查。此模块通过不等于这些依赖已闭合，不等于全包 IMPLEMENTATION_READY=YES。本模块r1独立审查通过167项检查、0差异、无新增必要局部问题；见test-vectors/OAR-FRONTEND-CLOCK-LOCAL-REVIEW-STATUS.json。
