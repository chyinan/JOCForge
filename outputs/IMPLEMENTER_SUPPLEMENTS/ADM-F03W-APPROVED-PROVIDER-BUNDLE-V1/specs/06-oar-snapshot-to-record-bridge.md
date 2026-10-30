# F06OB：snapshot、时间头、持久 record 的连接

## F06OB.1 输入域与尚未支持的分支

输入 snapshot 使用 F06QS.1 的完整 owned snapshot；N=source_capacity 为1～159，F=subframe_capacity为1～8。integer 均指数学整数，排除 boolean，不做浮点或字符串转换。所有 hex 遵守 F06QS 的严格 ASCII、精确长度协议。操作先检查所用字段，再写输出；输出拥有独立存储，不修改输入。非法形状、类型、长度、索引或容量返回 INVALID_BRIDGE_INPUT。这是隔离接口错误，不认证原生错误后的状态或异常身份。

本条款支持 auxiliary_profile=0。非零值返回 UNBOUND_AUXILIARY_PROFILE，不能忽略辅助源或用 object 替代。preset_present 非零且 preset_hex 首字节非零时返回 UNBOUND_PRESET_PROFILE；该分支会改变 header/record 的生产规则，仍须补齐。preset 不存在或存在且首字节为0属于当前域，其余24字节在这些运算中不消费。首字节0不是“全部 preset 不存在”的同义词，也不是允许用全零替换真实 preset。

本条款连接明确的 payload 数值接口，不认证实际 task/global source 路由、producer 默认配置、owner registry 或运行代次。所选路线必须证明其真实对象满足上述分支条件，不能从测试用例声明推导。

## F06OB.2 timing36 和 Source30 的写入与寻址

timing_hex 为18个U16LE，共36字节：word0=start_offset，word1=entry_count，word(2+2j)=start_units，word(3+2j)=full_ramp_samples，j=0～entry_count−1。entry_count须为0～F；每对值本身为U16。其余U16保留，不加入有效条目，不补零、不排序。解码产生 timing_fields={start_offset,entries}；它是时间字段投影，不能直接冒充F06QT.1要求的完整frame。start_units仍是32样本单位，full_ramp_samples仍是完整时长，二者不能互换。

适配到F06QT.1时另按 .3 从同一snapshot的description计算 materializer_source_count，再组成 frame={start_offset,entries,materializer_source_count}。第三字段不是timing36中的编码字段，也不是从expected或active_source_count填入的值。本域的description行数不随已选择子帧j变化，因此可在时间选择前计算该值；无到期条目时仍按F06QT返回source_count0。此适配只适用于已通过 .1/.3容量和分支校验的snapshot。

每个 sources[i].records_hex 的分配长度为30F字节。子帧j的 Source30 是字节 [30j,30(j+1))；i属于[0,N)，j属于[0,F)。选择一个子帧不重新解释或拼接其他source行。

timing_write(snapshot,timing_hex) 校验并复制完整36字节；snapshot其他字段保留，包括所有存在条件。source_write(snapshot,i,j,source30_hex) 校验并复制恰30字节到该行/子帧，同时将state_words[3]置1；其余state_words、active_source_count、description、descriptor tag/kind、其他子帧、其他源、可选payload均保留。校验失败不输出正常snapshot。生产者多步调用的跨阶段回滚尚未由这两个原语定义。

## F06OB.3 description 驱动的输出行数和 coded selector

从description48按F06OD.1解码object_count、group_count（0～9）、group_masks和auxiliary_profile。按group_masks存储顺序遍历每组，再按bit0～16递增枚举置位。每个置位输出一行，selector按F06OS为bit+1；组间重复位不去重。bit17～31不生成coded行，但完整mask仍参与header OR。

coded行之后生成object_count行。输出source_count是上述coded行数加object_count，须≤N；它由description决定，不能直接替换为snapshot.active_source_count。materializer读取 sources[0..source_count) 对应的已分配body。

此consumer不检查state_words[3]，也不使用descriptor.tag/kind来重新排序。F06QS在source_data_present=0时保留目的body，materializer仍按复制后的description读这些保留值；不能把“复制时不写body”误解为“解码时不读body”。真实producer的description/body一致性和失效生命周期仍需独立证明。容量校验仅避免越界，不替代该证明。

## F06OB.4 完整 record60 的覆写和保留

输入subframe j和record_seed，为全部N槽、每槽15×I32；所有槽均在写入前检查。每个种子是该current record槽的前值，不是期望输出。cold种子由F06OE.4清零规则产生；后续调用须保存前次自身生成的完整值。输出全部N槽，未生成的尾槽不写。

每个生成行使用 .2 的Source30。coded行覆写word0=1、word1=selector，word2～11先保留。object行先计算 z=(byte13!=0 ? I16@10 : 0)，component映射byte12 {1:3,2:2,3:4,4:1,5:5,6:6}，其他0。

若byte14==0，或byte27==1，或I16@28>0，生成type2，覆写word0～7为 [2,I16@6,I16@8,z,component,U8@27,U8@13,I16@28]，word8～11先保留。

否则生成type3，覆写word0～8为 [3,I16@6,I16@8,z,I16@16,extent_a,extent_b,component,U8@13]；byte14==1时extent_a=extent_b=I16@16，否则分别为I16@18、I16@20。word9=Wrap32(I16@4+512)>>10，右移为无符号逻辑右移，word10～11先保留。type3记录的生产已定义，但其完整spread音频目标不因此已实现或验收。

所有生成行再覆写word12=(byte0==0)，word13=I16@2，word14=(state_words[5]!=0 ? additional.reset_map[i] : 0)。word14在此是原U8值转I32，不自动正规化为bool；真实setter产生0/1见F06OE.2。active/inactive均执行上述写入，不能在word12=0时跳过record生成。

最后执行 .5 的extension overlay。上述word保留规则与F06OM.4从零开始生成的合成canonical record属于不同接口：生产持久current槽不可每次使用canonical零种子。完整60字节比较见F06OE.4；何时提交previous cache由processor连接另行规定。

## F06OB.5 snapshot→存在条件与 extension overlay

将state_words[5]!=0投影为additional_present，将state_words[7]!=0投影为frame_extension_present；两者独立。additional不存在时word14写0，不消费reset_map。additional存在时消费task-local索引i的reset_map；reset_map切片到global source的路由仍属BND001。

extension不存在时，全部生成行word10=word11=1，payload保留值不消费。存在时，对每个行i和子帧j：packed=frame_extension.flags[2i+floor(j/4)]；mode=(packed>>(2(j mod4)))&3；flags=frame_extension.modes[i]；word10=((flags&(1<<j))==0)；word11=0（mode0）、1（mode1）、2（mode2或3）。两个载体的字段名不改变此消费方式：flags数组承载2-bit mode，modes数组承载逐子帧布尔位。

type5～10的辅助行会共享首次该类行的overlay值；.1拒绝辅助profile，因此本域不会产生该类行。本域中各coded/type2/type3行独立消费自身i。此overlay定义扩展状态到record的写入，不认证其上游writer或point/spread renderer对所有mode的支持。

## F06OB.6 header80 的生成及保留

输入header_seed为80×I32，完整检查，输出保留未覆写词。word0=description.direct_mode（U8转I32）；word1=所有有效group_masks的按位OR，以32bit二进制重解释为I32；word58=0。

additional不存在时，word2=1、word3～12=0，其余word13～57和59～79保留种子。

additional存在时：word2=additional.mode为1/2/3时的该值，否则0。按cell index0～8处理：additional.enabled[i]非零表示该cell不参与，header.word(4+i)=0，五个cell词保留；值为0时header.word(4+i)=1，并覆写word(13+5i)..word(17+5i)。前三项来自该cell五字节的前3个U8，映射 v<16→v+1，v==16→0，v>16→16；后两项将后2字节重解释为I8并扩展为I32。

word3=0若九cell均不参与；否则九cell全部参与且原五字节均等于[16,16,16,0,0]时word3=1；其他组合word3=2。word59～79仍保留。字段名enabled是复制载体名，其非零值在此consumer中表示不参与，不能按名称倒置这个条件。单cell语义和实际warp/default生成须按已验证producer连接，不能用本条款臆定原始表。

## F06OB.7 timed materializer 的连接顺序

输入query为U32，initial_cells为两个U32，header_seed和record_seed如上。先按 .2 解码frame，再执行F06QT.3：从第一项扫描，遇未来start立即停止，选最后已到期项；不排序。无选中项时source_count=0，cells、header、records均保持caller输入。

选中j后，先更新cells=[max(0,full−elapsed),full]，elapsed=query−start_offset−32×start_units；再用 .6 生成header，用 .3～4生成records，再执行 .5 overlay。source_count为description产生的行数。source_count==0时，cells及header仍可能已更新；调用F06QT.4的queue fallback须传递这些自身生成值，不能重置或从fixture取下一帧状态。成功与零count均不复制current到previous；此提交由上层processor负责。

## F06OB.8 验收、连接状态与限制

OAR-SNAPSHOT-BRIDGE-VERIFICATION.json：96个cases提供完整snapshot、subframe、header_seed、record_seed及期望；50个timed_cases提供query和initial_cells，含空条目、非零start_offset、存储顺序停止、重复start和full0。writer_trace从writer_seed开始，逐步调用timing/source writer，用自己生成的前状态推进；禁止expected_snapshot作为下一步seed。9个rejection_cases按operation选择接口，验证错误标签及输入未修改。

96+50个case及10步writer序列与受限完整编译header/body/time/accessor/writer比较一致，无leaf substitution。枚举输入由2^bit及cell_index的数学规则生成，并与有界原资产逐项核验；未搬运原表至本包，未加载原DLL或读取原PCM/私有key。

无继承研究上下文的r2隔离复审已通过96 header/body、50 timed、10 writer、9拒绝及215补充检查，零差异。OB-DOC-01（timing_fields与完整QT frame的第三字段归属）已由 .2/.3 的显式适配关闭，零新增必要问题。详见OAR-SNAPSHOT-BRIDGE-LOCAL-REVIEW-STATUS.json；向量内审查字段是生成时历史状态。该审查未接触编译原件，不能独立认证native identity、实际producer、owner/generation或callback。

BND003/004/005/007的上述payload接口已补充；实际description/default/body writer链、global/local源索引、运行owner/generation、完整cache提交与callback控制仍未由这些局部用例证明。辅助profile和非零preset首字节为明确缺口，不能宣布8项连接全闭合。仅内部研究/实现，本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
