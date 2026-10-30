# F13 EMDF 与元数据时间

Owner override：F13 是 ID11/ID1/ID2/ID14 四个 FIFO、ID1 timer 和 EMDF assembly state 的唯一 executor/receipt/commit owner。上游 F05L/F05JL 只提供 immutable body，不得 push/pop 或保存 queue cursor。来源见 `00-selected-owner-projection-and-j05.md`。

当前产品目标依TARGET-AND-QUALITY-ACCEPTANCE。旧primary4/secondary1的HMAC含义只属历史研究路线；当前无私有key候选使用F14K。封装模式的显式开关见F13.9。

状态：`PARTIAL_SELECTED_TIMELINE_LOCAL_REVIEW_PASS`。容器version0/key0、FIFO、ID1 timer/wire、六个生产周期、四payload顺序和JOC complexity已定义；相关selected timeline的r1 package-only审查零发现。ID11 live producer由F05L定义，ID14由F05JL定义；F05JL r3与F05L r5 clean局部组合复审已通过；ID2/ID1同时ready barrier及真实冷ADM整链仍有Q08缺口。精确规则和参数见[F13JO](13-joc-complexity-timer-payload-order.md)。

## F13.1 容器

容器前四字节为 `58 38` 加上大端 U16 body byte length。body 从 version 2 bits=0、key_id 3 bits=0 开始。payload 按请求提供的顺序序列化，允许 ID1～30。当前实际顺序固定为 ID11、ID14、ID2、ID1；不能按数值 ID 排序，也不能使用 queue 遍历顺序替代序列化顺序。

每项依次写 5-bit ID、配置、variable length（组宽 8，最多 2 组）、payload bytes。末尾写 5-bit ID0，然后 primary length code=2（4 bytes）、secondary length code=1（1 byte），随后保护 bytes，最后以 0 补齐到字节。

length 不计前四字节；保留位和字节 padding 参与 body 认证。容器总长度还应不超过 EC3 carrier 的 9-bit length 可表示范围，超范围帧失败。

## F13.2 variable unsigned

给定非负整数 v、组宽 w、最大组数 m。从低组开始取得 `v mod2^w`；若 v>=2^w，则令 `v=floor(v/2^w)-1`，继续取得下一组，直到 v<2^w。超过 m 组失败。将取得的组逆序写出，每组 w 个数据 bits 后跟 1-bit continuation；除最后一组之外为 1，最后为 0。

因此数据串的逆向解释中，接续下一组时先把既有值加 1，再左移 w，加下组值。禁止当作普通 unsigned LEB128。

## F13.3 配置

按此顺序写：sample-offset-presence（存在时 U11 offset 和当前固定的 1-bit extension=0）；duration-presence（存在时 variable unsigned，w=11/m=2）；group-ID-presence（存在时 w=2/m=8）；codec-data-presence（当前=false）；discard-unknown。

后续字段整体受 discard=false 控制：仅当 discard=false 且 offset absent 时，写 frame-aligned flag；aligned=true 时再写 create/remove duplicate 两 flags。仅当 discard=false 且（offset present 或 aligned=true）时，写 priority U5 和 processing-allowed U2。discard=true 时这些字段全部省略，包括 offset present 的情况。

配置 fixture 必须给出完整对象：sample_offset、duration、group_id 以 null 表示字段不存在；codec_data_present/discard_unknown_payload 为显式 bool。payload_frame_aligned、create_duplicate、remove_duplicate、priority、processing_allowed 根据上述条件为显式值，未被消费者可以为 null。不得从 expected container 反推缺省输入。

OAMD payload ID11 当前group=0、frame-aligned=false；JOC ID14当前group=0、frame-aligned=true、create/remove=false、priority=0、processing-allowed=0。ID11 body来自F05L，ID14 body来自F05JL；不能从reference copy。

## F13.4 不同延迟

当前 pipeline input delay 和 extra control delay 都为 0。普通输出 gate 与基础 metadata delay 为 3072 样本。ID11、ID2、ID1 的请求 delay 各为 4608 样本；ID14 请求 delay 为 3072。完整 FIFO 算子见 F13.5。

当前各 FIFO 数据容量为 ID11 160 bytes、ID1 10 bytes、ID2 8 bytes、ID14 543 bytes。容量是接口约束，不是 payload 的固定内容长度；推入长度由当次 producer 给出。

JOC 成功 sequence 状态初始化 counter=-1、成功计数=0；输出为 `max(counter,0)`，接受后 counter 原为 1023 时变 1，否则加 1。初次两次成功的 sequence 都为 0，此后为 1、2 等。失败不推进。此 sequence 只描述 JOC 成功调用；programme 失败后采用 I03 fail-stop，不能据此推断整个前端回滚。

## F13.5 FIFO 的状态、先写后读与所有权

单个 FIFO 的容量 C（bytes）、延迟 D（samples）和帧长 F（samples）均为正整数。残量 R=D mod F，槽数 M=ceil(D/F)，默认写槽 W=1若R非零，否则0。冷状态含 M 个空槽及 buffer IDs=[0,1,…,M-1]。buffer ID 标识可复用存储，不是 payload 代次。

一次 push 输入 tag、payload bytes、started bool 和 phase samples。长度范围0～C；started=false时本次phase按0处理，started=true时phase必须属于0～F-1。选择槽为 W 若R+phase<F，否则0；该槽必须为空。将 payload 与 tag 保存到该槽，新phase=(R+phase) mod F，新started=(新phase!=0)。返回写入槽号、该槽当前buffer ID和新phase/started。长度越界或目的槽占用时 push 不改槽内容；programme 仍按 I03 终止。

随后 pop 读取最后一槽及其buffer ID，结果可以为空。全部槽更新为[空,旧槽0,…,旧槽M-2]，buffer IDs更新为[旧末ID,旧ID0,…,旧IDM-2]。弹出的存储被回收到首槽；消费者必须在再次写入前取得自己的有效内容，不能保存可被覆盖的引用而当作永久历史。

当前 producer 每次输入 started=false、phase=0，F=1536。普通三个 FIFO 各3槽，JOC各2槽，残量均0。第一次 queue mutation 前，ID11、ID1、ID2、ID14 四个 body 必须全部成功并绑定当前 writer generation；任一 producer 失败时四个 queue 都不得改变。研究模型把 JOC、OAMD、ID2 作为预先生成的输入，因此没有观察这三个上游算法的调用先后；实现可顺序或并行计算，但不得越过上述 readiness barrier。

同一 codec source call 的 queue 顺序固定为 ID11、ID1、ID2、ID14。每个 ID 先 push 再立即 pop，之后才处理下一个 ID；四个 queue 全部完成后才执行 publication gate。source s的普通 pop writer=s-2（s<2时为空），JOC pop writer=s-1（s<1时为空）。source0/1不发布；source1弹出的JOC writer0不进入AU。published AU j在source j+2产生，消费普通writer j与JOC writer j+1。弹出后按 ID11、ID14、ID2、ID1 的顺序序列化 EMDF。

front callback零基编号为source s+2。这组关系只建立 producer/consumer 代次，不推导 waveform sample start。

最后source处理完成后不为排空元数据而追加codec frame。对本歌曲4249 calls，普通队列尚有writer4247、4248，JOC队列尚有writer4248；这些pending payload不发布。reset丢弃pending内容，恢复空槽和初始buffer IDs。

## F13.6 ID1 半dB编码和输入配置

输入三个完整 programme 的测量F32：dialogue loudness、ITU loudness、true peak，以及显式preferred bool。测量来自F06完整测量prepass，不能用当前source之前的滚动prefix值。

半dB算子：有限输入x先R32；若R32(x+120)<=2^-23，返回absence=-512。否则令u=R32(R32(x+x)+a)，a在x<0时为-0.5，否则+0.5；对u向零截断为整数，再限制到 loudness[-116,11]或true peak[-232,23]。这不是最近偶数的整数舍入。

9项配置按索引为[-2,1,int(preferred),0,half(ITU),half(dialogue),-512,half(true_peak),-512]。所有项为整数。字段6的absence来自当前无独立该项测量分支。该配置在各source调用使用，timer决定本次实际发送的字段。

## F13.7 参数化六timer

必要参数为 period_samples[6]：六个正整数，以48k sample为单位。`parameters/joc-timer-order-v1.json` 供应 `[24000,48000,48000,4800,24000,48000]`，依 timer index 0..5 排列。缺失返回PARAMETER_UNAVAILABLE。不能把“96-call”观察窗口解释为周期，也不能用合成测试周期产生生产payload。参数身份、字段映射和连续验收见 F13JO。

状态为 remaining[6]整数、cached[6]整数、mode=1、counter=-513。冷remaining=period_samples，cached=[-1,-1024,-1024,-1024,-1024,-1024]。当前只定义mode1/counter-513分支；每个source call（含priming）使用1536 samples步长。

先将六个remaining各减1536。对输入配置字段0：值为-2或-1且cached0为-1时，remaining0重新设为period0；其他值保存到cached0。

timer/输入field对应为(1,5)、(2,4)、(3,6)、(4,7)、(5,8)。当输入为-1024或-512且该cached仍为-1024时，remaining重新设为本timer周期；输入为这两个sentinel时不更新cached。其他输入通常替换cached；只有timer4使用严格最大值更新，输入大于旧cached才替换。

工作输出先复制原9项配置，再设field0=-2、fields4～8=-512。按timer优先级[1,2,5,0,4,3]找第一项remaining<=0：把其cached值写到对应field[5,4,8,0,7,6]；重置cached（timer0为-1，其余为-1024）和本remaining=本周期，然后停止扫描。一call最多刷新一项；其余到期timer保留非正remaining，等待后续call。字段1～3保持输入值。

输出为9项整数配置、完整next state、refreshed_field（无到期时null）。状态由本timer拥有，不能与FIFO的phase或JOC sequence共用。reset恢复上述冷状态；period取自已确认参数供应者，不由输出反求。

## F13.8 ID1 位流

按MSB顺序写version U2=0、field0>=0的presence U1；存在时写field0 U3。随后field1 U4；field1!=0时写field2 U1和field3 U1。

字段4/5的范围[-116,11]、宽7；6/7范围[-232,23]、宽8；8范围[-15,15]、宽5。依字段顺序写presence=(value!=-512)，存在时将value限制到其范围后减下界，以对应unsigned宽度写入。sentinel-1024若被timer发出，按“存在并clamp”处理，不等同于-512。

当前counter分支的最后flag=0，再以0补齐到byte。返回bytes及实际长度，不固定为10 bytes；10是FIFO容量。越界mode/field1/单bit输入或无合法生产周期时失败。ID1完整测量→配置→timer→wire→FIFO的每层测试分别标明范围，不能以半dB原语通过宣称测量闭合。

## F13.9 EMDF封装模式开关

ID2 的同代 Type11/LFE、外部 DRC profile 映射及完整 body 字段见[F13D2](13-id2-profile-metadata.md)。Type11 producer见F05L，DRC producer仍按F06条款单独证明。

配置名emdf_carriage_mode，字符串枚举deployed_compat/etsi_emdf。调用者省略时默认deployed_compat；显式null、空串、bool或未知枚举均返回UNSUPPORTED_EMDF_CARRIAGE_MODE，在帧计算前拒绝。不从文件扩展名、payload内容、参考输出或decoder报错自动改模式。

该开关仅作用ID11/14的以下配置；ID1/2按各自已定义producer配置，payload bodies、序列、FIFO时钟与JOC/OAMD语义不因开关改变。

| 字段 | deployed_compat默认 | etsi_emdf实验 |
|---|---|---|
| sample_offset/duration | null/null | null/null |
| group_id，ID11/14相同 | 0 | 0 |
| codec_data_present | false | true，并写reserved U8=0 |
| discard_unknown_payload | false | false |
| ID11 payload_frame_aligned | false | true |
| ID14 payload_frame_aligned | true | true |
| create_duplicate/remove_duplicate | ID11不消费，ID14为false/false | 两ID均false/false |
| priority/processing_allowed | ID11不消费，ID14为0/0 | 两ID均0/0 |

不消费字段以null在配置接口表示，不额外写bits。开启codec_data_present必须在presence后紧跟8个reserved零bits，再写discard flag；不能只翻转一个bit而保持旧后续位置。

etsi_emdf采用P06 Table41的选定字段要求。当前MediaInfo解析器的EMDF sync/coherency识别要求codecdatae=0；本机26.05实际拒绝codecdatae=1候选的JOC识别。因此默认沿用已验证播放器兼容配置，同时保留实验选项。这里不宣称“主保护长度0合法”，也不因实验模式而恢复私有HMAC。

切换模式必须先重新完整序列化EMDF、取得实际bytes，再执行F12码流预算/试算。header/skip length、carrier预算、body后位置和EAC3 CRC都按新长度生成。跨旧/新配置不得复用旧byte位置、旧CRC或旧BAP搜索结果。生产programme开始后固定mode，reset后可重新选择；实时中途切换及decoder seek重启尚未定义，拒绝该行为。

两个模式均使用F14K primary4/secondary1，checksum算法、尾填充策略与模式选择正交。UI建议“播放器兼容（默认）”和“ETSI EMDF封装（实验）”。禁止把后者标为“整个码流ETSI严格符合”；旧OAMD warp3在严格body验证仍失败。完整body、播放器显示JOC、实际空间render及商用decoder兼容分别验收。
