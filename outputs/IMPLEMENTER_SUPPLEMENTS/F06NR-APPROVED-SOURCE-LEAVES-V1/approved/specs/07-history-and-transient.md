# F07 输入历史与瞬态

状态：`SELECTED_RULES_AND_ACTUAL_PARAMETERS_SUPPLIED`。生产参数D05的33个F32槽位由 `parameters/transient-d05-authorized-v1.json` 供应；实际参数r3局部复审已通过，整曲生产等价仍未由本局部结果闭合。

## F07.1 六路输入历史

状态是每路 1280 个 binary32 样本，冷初始化全部 positive zero。该状态与 F08 的每路 128 样本折叠状态分开。

对每个 256 样本块，先输出旧历史的样本下标 `[256,512)`；再移除旧历史最早 256 个样本，把当前块的 256 个样本附加到尾部。六个块从 0 到 5 顺序执行，六路独立。输出样本位模式完全按字节复制，不做数值运算。该规则产生 1024 样本的输入延迟，历史本身仍为 1280 样本。

一帧后输出 6×1536 个样本及更新后的 6×1280 状态。输入长度错误、状态长度错误拒绝并保留原状态。普通接口分帧时不重新冷初始化。

## F07.2 瞬态状态与资产

每个全带宽通道有：两节滤波的 4 个 F32 历史、三节滤波的 6 个 F32 历史、18 个 F32 peak statistics、前一块 mean、4 个 short 队列槽和独立的 4 个 other 队列槽。冷初始化为正零和 0。

producer的输入是未经过F07.1的 `installed_audio`。不能使用delayed audio然后再次排队。滤波器需要两节10系数、三节15系数、short三阈值、other三阈值和minimum power；全部由授权provider按F07A字段顺序供应，不能由实现者挑选相似滤波器。

## F07.2a 两节滤波

每节系数定义为 `[a0,a1,b0,b1,b2]`，历史为 `[h0,h1]`。按样本递增顺序，输出 `y=(x⊗b0)⊕h0`；新历史为 `[( (x⊗b1)⊕h1 )⊕(y⊗a0),(x⊗b2)⊕(y⊗a1)]`。第二节使用第一节的 y。每节的新历史在该样本计算结束后写入，下一样本使用新历史。

两节结束后，对4个历史逐项abs并从F32正零递增求和。仅当sum!=0且sum<clear_threshold时将整组历史清为正零；sum=0时保留原零符号。clear_threshold是授权provider中的D05-C；provider文件或字段缺失报 `PARAMETER_UNAVAILABLE`，字段类型、位模式、顺序或hash不符报 `PARAMETER_IDENTITY_ERROR`。清零发生在块末，不能逐样本清。

## F07.2b 三节成对滤波

此阶段消费两节滤波输出。每节仍按 `[a0,a1,b0,b1,b2]` 排列，历史 `[h0,h1]`。定义 `q=(b0⊗a0)⊕b1`、`z=(b0⊗a1)⊕b2` 和四项向量：

- `U=[1,a0,(a0⊗a0)⊕a1,a1⊗a0]`。
- `V=[0,1,a0,a1]`。
- `P=[b0,q,(q⊗a0)⊕z,q⊗a1]`。
- `Q=[0,b0,q,z]`。

每对输入 `[x0,x1]` 对每 j=0～3 计算 `v[j]=(((x1⊗Q[j])⊕(x0⊗P[j]))⊕(h0⊗U[j]))⊕(h1⊗V[j])`。输出前两项，后两项为下一对历史。三节按节递增执行，每节处理整块后输出交给下一节。

块末按 F07.2a 同样的 abs-sum 顺序及门限处理全部 6 个历史。不得替换为逐样本普通 IIR，成对计算的舍入顺序是接口的一部分。

## F07.2c Peak 和标志

当前 mean 用 F09G.2 的四累加器规则，基于未滤波 PCM。滤波结果分成四段各 64 样本，每段取 abs 最大值并夹到 [0,1]；相等时选择最后一个位置，位置为整块 0～255 下标。四段峰值写入 statistics[14:18]；两个半块峰值写入 [8:10]；全块峰值写入 [2]。其余统计项暂保持旧值。

对 level=0/1/2，分别 count=1/2/4，base=6×level，tier 为存在 j∈[0,count) 满足 `statistics[base+j+2]⊗threshold[level] > statistics[base+j+1]`。short 使用 short thresholds，只在全块 tier 为真时可为 1；minimum>statistics[2] 或 previous_mean>R32(0.5)⊗mean 时清 0。

other 使用独立 thresholds，对三层 tier 取 OR；minimum>statistics[2] 或 previous_mean>mean 时清 0。它读取同次滤波结果和处理前历史，不读取已经更新的 short statistics。

判定后依次令 statistics[0:2]=原 [1:3]、[6:8]=当前 [8:10]、[12:14]=当前 [16:18]，保存 mean。这里各右值在该次块更新前先读取；当前峰值项保留到下一块覆盖。short/other 队列按 F07.3 推进。

## F07.3 消费队列

每块读取 old queue 第 0 槽作为当块消费标志，然后左移其余 3 槽，把本块 producer 标志写到第 3 槽。short 和 other 分别操作。首次 4 块消费标志均为 0。

other 由同次未延迟输入、同次滤波输出及处理前统计历史产生。已经得到的 `other_consumed` 直接参加结构刷新，不能再延迟 4 块。

## F07.4 结构刷新

每个全带宽通道的第 0 块刷新标志为 1。块 `b=1..5` 的标志为以下条件的逻辑或：当前 other_consumed=1、当前 short_consumed 与前一块不同、当前 SPX 标志与前一块不同、当前 end-bin 数与前一块不同。

比较的是当前 other 本身，不是当前/前块 other 是否不同。本 profile 的 SPX 全部为 0，end-bin 全部为 217。

## F07.5 Dither

每通道携带前一块的 short 标志，programme 冷启动为 false。当且仅当当前和前一块都不是 short，该块 dither flag 为 true。先计算该值，再更新 previous_short；跨帧保留第 5 块标志。

当前自动 selector=7、acmod=7、无 coupling，配置屏蔽不改变上述 flag。header 的 dither syntax 当任一 flag=false 时存在；全 true 时省略，省略语义为 implicit ON。flag 推进发生在接受帧时，失败试算不提交。

其他 selector 的 mask 分支未纳入当前端到端支持范围，不要求实现者推断。

## F07.6 输入、行顺序和状态所有权

一帧输入包含 `installed_audio` 与 `delayed_audio`，各六行，每行 1536 个有限 F32。外部行顺序为 `[L,R,C,LFE,Ls,Rs]`。瞬态和谱分析的 FBW 行顺序为 `[L,C,R,Ls,Rs]`，对应外部下标 `[0,2,1,4,5]`。LFE 是外部下标 3。

`installed_audio` 是此前已完成安装的普通六路输出，不是原始 ADM 38 路。`delayed_audio` 必须是同一源帧经过 F07.1 后的输出。不能拿延迟输出再次生成瞬态，也不能绕过输入历史。配置中的 sample format=0、analysis enable=0、configuration mode=15、extra mode=1、control0=1、control1=0，是本固定分支的前置条件；不得把它们泛化到其他配置。

每个 FBW 通道独立保存：two_history[4]、three_history[6]、peak_statistics[18]、previous_mean、short_queue[4]、other_queue[4]。滤波和统计项为 F32，队列项为 0/1。两个队列共享本次滤波输出和处理前统计状态，不共享队列存储。other 不需要第二套滤波历史或第二套 peak_statistics。

整个 programme 还保存六路 1280 样本的输入历史、各 FBW 两个独立 128 样本折叠历史、LFE 两个独立 128 样本折叠历史，以及五个跨帧 previous_short 布尔值。全部冷初始化为正零或 0。不能将构造器的零初始化当成每帧行为。

## F07.7 每帧固定执行连接

先完成 F07.1 的字节复制，得到当前帧 `delayed_audio` 和六路 next input history。瞬态仍读取 `installed_audio`，两者不能混用。

按 block=0～5、FBW lane=0～4 处理各 256 样本块。F07.2 的 mean 来自本块未滤波 installed PCM。先两节滤波及块末清零，再三节成对滤波及块末清零，得到本块 filtered[256]。short 和 other 都读取这一次 filtered 和调用前 peak_statistics、previous_mean。short 的全块 tier 必须为真；other 对三层 tier 取 OR，不要求全块 tier 为真。双方各用自己的三阈值。不得先提交 short 的统计后再计算 other。

两种 producer flag 计算完成后，各读取自己的 old queue[0] 作为 consumed flag，左移队列并写入 producer flag。统计历史只更新一次，previous_mean 只写入一次。输出形状分别是 short_consumed[6][5] 和 other_consumed[6][5]；外层是 block，内层是 `[L,C,R,Ls,Rs]`。这些值不是源帧是否发布的标志。

历史原型 JSON 的 `consumed_flags[source].long_flags` 名字易混淆：它逐项保存 old short_queue[0]，实际就是这里的 short_consumed。值 1 表示选择短变换，值 0 表示长变换。adapter 直接将该 0/1 值转换为 short 布尔值，**不取反**。普通主变换、companion 及 F06ND 的 secondary 变换都读取这个同源帧、同 block/lane 标志；other_consumed 不替代它。

对每个 FBW，取该通道六个 consumed short、other、六个 SPX=0 和 end-bin=217，按 F07.4 产生结构刷新六项。第 0 块每帧强制为 1；其余块使用本帧相邻块比较。结构刷新比较不读取上帧第 5 块；dither 的 previous_short 才跨帧。

对 block=0～5、FBW lane=0～4，F08 读取当前 delayed PCM 块、该通道 primary/companion 的旧折叠状态、同块 short_consumed。flag=0 时两个 bank 都调用长路；flag=1 时两个 bank 都调用短路。两条路径都更新各自 128 项状态，切换长短不清历史。Primary 短谱和 companion 短谱均按 F08.6 交错输出，不能将 companion 改用长路。

两 bank 的前 217 项构成 POWER：`min(1,R32(R32(primary[k]*primary[k])+R32(companion[k]*companion[k])))`。PRIMARY 原始指数仅消费 primary 的前 217 项。LFE 每块分别执行 F08.7～F08.9 的 primary、companion 七项路径，不读取 FBW short flag。结构刷新参与 POWER 指数刷新及主指数计划；converter 再消费已保存的 POWER 指数及结构标志。converter 处理内部派生行时不重新运行 ADM PCM 瞬态分析，不新建 42 路 F07 状态。

原型的计划阶段与后续编码阶段分别拥有自己的 primary 折叠状态。它们从同一冷状态和同一 delayed PCM、short flags 重建。隔离实现若将已计算 primary 谱交给编码器，应确保实际消费者读取同一次谱结果；不能在同一状态上再次执行 primary 变换，导致一个块推进两次历史。共享缓存是否可以作为优化，需另有消费等价验证。

## F07.8 Dither 消费和接受帧提交

固定调用参数是 selector=7、acmod=7、coupling 六项均 false、gate=true。未选择其他 selector 的 mask 分支。本分支每个 block/lane 的 flag 为 `!current_short && !previous_short[lane]`；随后令该 lane 的 previous_short=current_short。当前 short 是四块延迟后的 consumed flag，不是 producer flag。

先计算整帧 30 个 dither flag 和下一帧五个 previous_short。帧 header 的 dither syntax 位为任一 flag=false。syntax=true 时每块按五个 FBW lane 写五个 flag；syntax=false 时省略所有 flag，解码语义为全 ON。无 LFE dither flag。省略 syntax 不能解释为全部 OFF。

编码器用 staged 状态计算变换、dither 及预算试算；预算重试复用本帧 dither 结果，不再次推进 previous_short。只在帧成功接受时提交 next state。预算尝试失败不提交。全帧最终失败按 I03 退休整条 programme，而不是回到已不匹配的上游状态继续。

## F07.9 Priming、EOF 和 reset

普通源帧下标从 0 开始，仅对上游已实际产出的完整六路帧编号。上游最初两个不产出六路帧的 frontend callback 不运行 F07，也不虚造六路正零来推进本模块；它们不等于 codec 的两个发布抑制源帧。所有保留的普通源帧都必须执行 F07 和 F08，即使前两帧尚未向文件发布。首帧不能预先设置队列或折叠状态为后续帧的值。输入历史产生的 1024 样本延迟、四块 flag 延迟及文件发布的两源帧 suppression 是不同机制，不能互相抵消。

已验证整首固定执行保留 4249 个普通源帧，前 2 帧完成编码后抑制发布，发布 4247 AU。上游结束规则由对应普通 frontend/输出时钟条款决定，本模块逐帧处理实际收到的 installed/delayed PCM。不要在 F07 EOF 补造一帧 PCM=0、清空瞬态队列，或对峰值状态另做 flush。当前完整 profile 的 codec additional flush=none。

显式 reset 重建 input history、全部滤波历史和统计、两个 consumed 队列、两 bank 折叠状态、LFE 两 bank 折叠状态及 five previous_short，同时重置上游时钟和所有关联 codec state。不能仅清 short_queue，也不能在源帧 2 发布开始时 reset。

## F07.10 参数供应者、输入错误和数值错误

供应者必须给出two[10]、three[15]、short_thresholds[3]、other_thresholds[3]、minimum[1]、clear_threshold[1]，合计33个F32槽位。每节按 `[a0,a1,b0,b1,b2]`。参数U32位模式的版本、长度、内容身份由 `parameters/transient-d05-authorized-v1.json` 绑定。三个aux thresholds未被选定caller消费，不属于本profile provider。实际参数合成验收不能代替整曲生产运行。

D05-P/A/S/O 来源是研究侧既有非秘密 compiled 音频静态参数记录；D05-M 是该记录的标量，但旧记录没有保留原始参数位置；D05-C 是已验证模型明确使用的阈值字面值。它们不是私有 key 或认证材料。用户已允许普通非秘密功能常数用于内部隔离交付与实现，33 个槽位已由 `parameters/transient-d05-authorized-v1.json` 供应；其中五组 biquad 的 b0/b2 重复，共 28 个不同功能值。完整原表、源码和二进制仍不属于本交付，也不据此声称公开来源或对外分发权限。

入口必须在计算前验证完整输入、state 和参数的形状、类型及有限性；本模块 flag、queue 与 previous_short 的外部编码固定为整数 0/1，拒绝语言级 bool、其他整数和字符串；进入内部布尔运算时再按 0=false、1=true 转换。参数不存在报 `PARAMETER_UNAVAILABLE`，身份不匹配报 `PARAMETER_IDENTITY_ERROR`，PCM/state 形状、类型或域错误报 `F07_INPUT_ERROR`。运算中产生非有限 F32 或溢出报 `F07_NUMERIC_FAILURE`。错误不返回正常 next state，按 I03 退休整条 pipeline；重新开始必须执行完整 reset。原型内部函数抛出的异常文案不是隔离接口协议。

## F07.11 本轮验证范围

实际完整原型的 4249 个计划源帧下标和选定消费者源身份已检查。参数化独立方程和原型以两个独立合成参数组连续处理 36 块，比较 10836 个滤波、状态、标志、峰位置等字段。五帧字节复制历史一致。dither 使用编译后的原型纯 Rust 函数核验 2048 个计划、73728 个布尔值；固定 SPX/end-bin 的结构刷新穷举 4096 个 short/other 组合，比较 24576 个值。全部零差异。

本轮没有重跑生产整首，没有读取DLL、SDK、私有key或native PCM。既有完整音频/位流结果的范围不因这些合成检查扩大。参数供应已闭合；隔离实现实际编码、空间播放和历史exe↔Rust build身份仍须独立完成。


验收材料：`test-vectors/TRANSIENT-SYNTHETIC-VERIFICATION.json`、`test-vectors/TRANSIENT-SYNTHETIC-BOUNDARIES.json` 和 `test-vectors/TRANSIENT-SELECTED-CONSUMER-RESULT.json`。参数化F07 r1与实际参数r3局部复审均已通过；本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。相关调用连接见 `PIPELINE-AND-INTERFACES.md` 的 I03/普通 source clock、F08、F09/F10、F12 及 F06ND。

F07参数化r1独立审查通过：36连续块、2048 dither计划、4096结构组合、5历史帧、5边界及12拒绝均无差异；未发现新增必要算法问题。见 `test-vectors/TRANSIENT-PARAMETERIZED-LOCAL-REVIEW-STATUS.json`。F08/POWER/converter/F06ND连接仅静态审查。实际33槽provider及连续验收见F07A；不把局部合成结果扩张为整曲生产验收。
