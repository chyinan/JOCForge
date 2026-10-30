# F06OR：选定16源路线的 task、索引与分类

## F06OR.1 范围与分区

本条款固定 global_source_count=16、global_direct_count=1、worker_cpu_count=16；上游F03R/I03提供16行SCP及16行control52，global source0为coded LFE，1～15为对象。这里的source index是该逻辑行序，不是ADM WAV物理track ordinal。其他来源数、direct count、CPU分区或layout不属于本条款支持范围，返回 UNSUPPORTED_ROUTING_PROFILE，不能套用本条款后宣称原路线完整。

路由入口的input恰为含task_index、global_source_count、global_direct_count、worker_cpu_count四键的对象。不是对象、缺键或多键先返回INVALID_ROUTING_INPUT。随后按global_source_count、global_direct_count、worker_cpu_count顺序校验：三个值须为数学整数（排除boolean/浮点/字符串/null）且分别等于16、1、16；否则UNSUPPORTED_ROUTING_PROFILE。再校验task_index，最后校验descriptors；混合错误以此顺序的第一个错误返回。所有拒绝均不产生正常输出、不改输入，不声明原生错误等价。

输入task_index为数学整数0～7，排除boolean；非法task index返回INVALID_TASK_INDEX。上述检查先于产生正常路由输出。选定factory分区宽度是ceil(16/max(1,16−1))=2；顺序创建8个task。task t 的start=2t、end=2t+2，end不包含。每task N=2、F=8，分别拥有自己的snapshot、reset_map、queue、current/previous records和gain rows。

## F06OR.2 唯一映射

对于local source r∈{0,1}，global source i=start+r。对应关系如下，所有入口保持同一i/r关系：

| 对象/入口 | 使用的索引 |
|---|---|
| shared metadata getter及F06OM.2的control52输入 | global i；direct_count=1 |
| 上游SCP的PCM逻辑行 | global i |
| 本task的PCM view行 | local r，对应上游行i |
| 本task metadata setter、Source30行、descriptor行 | local r |
| 本task additional.reset_map及extension载体 | local r |
| 本task current/previous record和gain row | local r |

task0的local direct_count=1，对象数1；task1～7的local direct_count=0，对象数2。shared getter计算metadata使用global i，不能改成local r/direct_count1。setter用local r定位行，然后用该descriptor.tag==2判断object_source；该值必须等于(i!=0)。descriptors是input之外的独立可选参数；省略或显式null都表示只生成标准路由、不验证实际descriptor。其他值须为两个对象组成的数组，每对象恰有tag/kind两个整数键（排除boolean），且逐行等于本条款的tag/kind；形状、类型或值失配均返回ROUTING_CLASSIFICATION_MISMATCH。不以输入PCM是否非零重新分类。

source0的descriptor为tag0/kind3；所有对象的descriptor为tag2/kind0。F06OD的任务description输入分别为（对象数1,coded_lfe_count1）和（对象数2,coded_lfe_count0）。mask8的coded descriptor kind3、record selector4、六路output的LFE索引3属于不同编号域。

F06OE中source_count16的显式合成状态可表示全局recipe，但生产接口以八个独立N2状态消费。若验收需要全局16项map投影，按i=2t+r拼接各task的map[r]；反向切片按同一公式。该验收投影不授予16项数组与八个owner共享可变存储的权限；复制或切片的实现必须保持每task写入不影响其他task。

## F06OR.3 metadata调用次序和PCM消费

每task先按i=start、start+1递增调用shared getter(i)，将所得metadata交给local setter(r=i−start)，最后调用该task processor。metadata getter的返回内容不是从其他task的metadata cache中预测。processor使用相同区间的PCM行和local Source30/records，不允许将按物理WAV顺序读取的两行直接当作这些逻辑行。

task wrapper还向setter转交一个独立U32上下文值；constructor初值为0。本路由条款只定义其转交，不把它命名为frame_samples或ramp duration，也不允许据此覆盖F06OM的frame_samples。实际getter中metadata.samples由其自己的frontend frame时钟提供，setter的自动ramp更新及其他上下文写入归状态/时钟producer条款。

八task输出六路顺序和有序累加使用F06M；全局source order与输出声道order不是同一索引。混音数值、样本时钟、callback事件及shared DV32不由本路由条款另行定义。

## F06OR.4 选定coded LFE到selector的供应关系

选定coded标签集合恰含一个LFE channel enum3。此功能枚举的mask转换为8；集合OR仍为8；F06OD的coded group count为1、group_masks[0]=8。F06OB按生成的bit0～16顺序枚举该mask，唯一coded bit为3；F06OS输出selector4，F06OB把它写到该coded行record.word1。这是mask供应链，不能直接把descriptor.kind传入F06OS。

coded record的type1/type11选择依据实际preset：不存在或存在但preset首字节0时为type1，存在且首字节非零的生产分支仍为UNBOUND_PRESET_PROFILE。description.direct_mode被独立复制到header.word0，不用它猜测selector或type11。

已核对的selected initializer在direct_count非零时启用preset并把其首字节写0；direct_count零时关闭存在条件。已核对的Source30/reset_map/timing setter不写preset字节；F06QS在存在条件非零时复制完整preset。由这些明确操作组成的已分配对象链可保留首字节0。若存在其他preset writer、重配置或外部snapshot输入，必须单独验证此条件，不能延用该局部不变量。其余24字节的完整默认含义和消费者仍属单独依赖，不以首字节0为由删掉payload。

## F06OR.5 验收与证据范围

OAR-TASK-ROUTING-VERIFICATION.json的8个cases从input计算start/end/local_direct_count和全部路由字段，再比较40次getter/setter/processor控制调用的顺序。控制次序仅包含getter global index、setter local index、返回metadata对应的global index和转交上下文，不传递运行地址。15个rejection_cases从各自input及可选descriptors执行上述root、profile、task及classification校验，包含缺/多键、null root、严格类型和混合错误优先级；无正常输出，不改输入，错误属于隔离接口，不认证原生错误身份。

研究侧已执行受限编译task loop并比较该40次调用；getter、setter和processor是显式合成外部接口，没有认证其payload或callee identity。另以无leaf substitution的编译LFE enum helper核验enum3→mask8，并以F06OB/OS的有界证据核验后续mask枚举和selector。上述局部证据不属于真实ADM冷生成或音频验收。

BND001/BND002的选定索引与selector供应合同已完成独立r2复审：8 task、16 routes、40次控制调用、15拒绝和145项补充检查全部通过，零差异。OR-T01（严格类型、root形状、null descriptors、错误优先级）已关闭，零新增必要局部问题，详见OAR-TASK-ROUTING-LOCAL-REVIEW-STATUS.json。向量中的独立审查字段是生成时的历史状态；审查者未见编译源或callee原件，不独立认证native身份。

实际上游SCP/metadata producer、所有setter上下文writer、对象owner/generation和callback控制仍须分别闭合；本条款不认证其他CPU路线或整包就绪。仅内部研究/实现，本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
