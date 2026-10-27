# F06ND 固定 none 的逐帧音频统计、limiter 和 DRC 控制

状态：`SELECTED_NONE_DRC_ACTUAL_PARAMETERS_LOCAL_REVIEW_PASS`。仅内部研究和内部实现。必要功能参数 `DRC_FILTER5`、`DRC_NORMALIZE3`、`DRC_QUANTIZE3` 和 `DRC_AUX_ROTATION2` 共13个非秘密F32功能值已获用户逐组明确内部交付和实现授权，完整身份见 `parameters/none-drc-authorized-v1.json`。本文保留当前none的完整音频统计与控制算法；secondary实际FFT/连续链与13+34参数的局部fresh审查均已通过；真实整曲消费仍须完成。没有采用占位参数。

## F06ND.1 域、帧代和连接

仅覆盖 48 kHz、1024 kbps、5.1、acmod=7、line/rf 两项 XML 均为 none、四项 Lo/Ro/Lt/Rt mix level 均为 −3 dB（code=4）、frame_flag=1、metadata override=0。none 映射到 core code=0，RF 和 line 的 profile 均为空；本域不读取 96-byte 配置曲线。其他 profile、mix level、override 或耦合/SPX 政策必须显式拒绝，不套当前表。

每一 source 代 g 从 0 连续递增，消费 primary、secondary 各 6×1536 个 finite F32，平面顺序 L,R,C,LFE,Ls,Rs。每帧恰好六个 256-sample block，不把 RF 控制当 AU 总数。包括 codec priming 的前两帧也必须执行。既有完整节目模型实际执行 4249 代，而物理 AU 为 4247；这些数量不互换。

primary 是 F07 ordinary input-history 输出，必须且只能延迟一次：冷启动六路 1280-word 正零 history；各 block 先输出旧 history 的 sample[256:512]，再 history←old[256:1280]接当前 block256。接受已延迟输入时不再运行这一 gather。该延迟是 1024 samples，不是 meter 的 3104 samples。

secondary 必须由同代 primary 的**未量化 spectrum**独立重构。帧号、每帧预热、carrier写入与overlap持续状态的完整连接见F06NR；其inverse算子已由F06NI/F06NF供应并通过局部复审。当前政策无 coupling、无 SPX，fullband end=217（bandwidth code48）。五路 fullband 分别有独立冷零 forward128、inverse128 history；每帧六块，实际 transient flag 按 block-major 的 6×5 输入选择 long/short forward；将 spectrum[217:256]清正零后运行 inverse、overlap。inverse 的 block0 是预热，block1..5 写 secondary 的 block0..4，block5 没有 secondary 统计读取；LFE 没有 secondary 消费。forward/inverse 数学及表供应须在对应 F07/F12 条款闭合，本条款定义连接，不能用最终解码 PCM 或 primary 代替 secondary。

接口另消费该完整 programme 的 F06 selected_loudness 所产生的整数 dialnorm d∈[1,31]。先按 F32 计算 n=R32(S16(−256d)×2^-15)=−d/128。没有 loudness 默认值。统计和 scheduler 的 dialnorm offset 必须同代、同 programme。不是每帧重新测量完整歌曲。

每步成功后返回下一 owned state、同代 RF U16 word/flag、六个 line U16 words/flags，以及下文审计投影。F12 BSI 只消费同代输出的高 8 位，不回推 U16；RF flag=1、六个 line flags=1 是成功路径规则。不得为省算法预设 RF byte255/line bytes0：高峰输入可触发非零压缩。

## F06ND.2 运算、参数与冷状态

R32 为 N00 的 binary32 最近偶数舍入，以下 ⊕、⊖、⊗、⊘ 分别每步 R32，无 FMA。所有 input/state float 在消费前转 F32；有限输入导致中间非有限仍按 I03 失败。S16 对 U16 二补码解释，Sat16 饱和到[−32768,32767]；RoundEven 是最近偶数整数舍入。数组、索引和遍历顺序固定。

本文所有 `clamp(v)` 均为 `R32(min(1,max(−1,R32(v))))`，先下限−1再上限+1；包括滤波输出、RMS sum、normalized level、peak_log 和 RF 最小指标。不能因变量名字是energy或peak而改为[0,1]或其他区间。滤波持续state不调用clamp。

secondary 的30个实际flag来自同代F07的 `short_consumed[block][lane]`，block=0..5、lane按L,C,R,Ls,Rs依次排列，对外输入槽为[0,2,1,4,5]。值0选择long，值1选择short；历史验收字段名 `long_flags` 指的就是这些short-consumed值，不取反、不使用short_new或other。primary已在F07.1延迟一次，secondary的forward/inverse各自持续历史不是第二次input-history gather；不可再加1024源延迟。

必要参数：

| 名称 | 类型及长度 | 实际用途 | 当前供应 |
|---|---|---|---|
| DRC_FILTER5 | [a0,a1,b0,b1,b2]，5×F32 | 五路 paired biquad | 已授权内部功能值供应，见none-drc-authorized-v1.json |
| DRC_NORMALIZE3 | [A,B,C]，3×F32 | 非负 RMS/peak 的 normalized 指标 | 已授权内部功能值供应，见none-drc-authorized-v1.json |
| DRC_QUANTIZE3 | [Q0,Q1,Q2]，3×F32 | RF/line packed-control 多项式 | 已授权内部功能值供应，见none-drc-authorized-v1.json |
| DRC_AUX_ROTATION2 | [P0,P1]，2×F32 | 第三种 auxiliary stereo peak | 已授权内部功能值供应，见none-drc-authorized-v1.json |
| dialnorm_offset | R32((31−d)/144) | 将 dialnorm 编入 normalized 指标 | 31/31 实际有效项逐词匹配，生成原理与验收可交付 |
| auxiliary_mix | N=R32(−sqrt(2)/4), V=1, S=−1/2 | 三种 downmix 的 code4 行 | 100/200 decimal-digit 生成一致，3/3 逐词匹配 |

数学生成供参见 `parameters/none-drc-math-v1.json`。sqrt 从精确值生成至 binary32，不能先用低精度近似。固定 profile 仅需上述三个 mix 值；原三张8项表及原125项 offset 表不交付。offset 原消费 index=−(Sat16(RoundEven(R32(min(0,n)×32768)))>>6)=4d，严格落在4..124，31个值全由该规则生成。n=0 的旧通用接口特例返回0，在当前 d∈[1,31] 域不出现。

统计 state：filter_pairs[5][2]正零，previous_primary_peak[6]正零，previous_secondary_peak[6]正零，previous_downmix_peak正零。共23个 F32。scheduler：line/rf 各{level=1,hold=0,log_gain=0}；line/rf limiter 各{gain=0,hold=0}；peak_logs[6]正零；generation=0。四个 hold 为U16，generation 为U64。状态每代连续；每次完整 programme/reset 才恢复冷态。

metadata override FIFO 冷态为4个全零 logical records，metadata_delay=3072、append_slot=3、frames_processed=0。支持当前 opaque category5、32-bit、count1 submission；此 category 与 EMDF ID1/ID11 编号不同。每代先 shift records[1:]接0，读新record0，consume/reset为0，processed+1。当前 category5 不写 DRC control FIFO，故 override始终0；category1需要独立 authored-control producer，本域拒绝。这些0由初始化和未写入分支产生，不来自测试输出。terminal_peak_scale=1 由 constructor 初始化，选定 helper mode7 在 writer 前返回，因此状态不更新。换域必须另行定义 writer，不能保留1推论。

## F06ND.3 五路 paired biquad 与 mean-square

统计内部 channel 顺序 L,C,R,Ls,Rs,LFE，对外槽映射[0,2,1,4,5,3]。每 block 五个 fullband 通道依次运行各自 filter state；LFE 不进滤波和 RMS，但进 primary peak。

给定 state(s0,s1)，先 q1=(b0⊗a0)⊕b1，q2=(b0⊗a1)⊕b2，四向量：

q25=[1,a0,(a0⊗a0)⊕a1,a1⊗a0]；q26=[0,1,a0,a1]；q27=[b0,q1,(q1⊗a0)⊕q2,q1⊗a1]；q28=[0,b0,q1,q2]。

按 sample pair (x0,x1) 扫描128次，各 lane j 的 t=((x1⊗q28[j])⊕(x0⊗q27[j]))⊕(s0⊗q25[j])，v[j]=t⊕(s1⊗q26[j])。输出 clamp(v0),clamp(v1)，clamp先max(−1)，后min(+1)；下一 state=(v2,v3)，**state不 clamp**。

256个输出的 mean-square 使用四个独立正零 accumulator；i=0,4,..252，lane j 累计 acc[j]=acc[j]⊕(y[i+j]⊗y[i+j])。归约先 acc0⊕0，再依次⊕acc1、⊕acc2、⊕acc3；最后⊘256。五路 energy 从第0路开始按内部顺序相加，不计 LFE、不重排、不树形归约。level=clamp(normalize(clamp(sum))⊕offset)。

normalize(v) 要求非负 finite F32。u为其U32 word，t=(2u) mod2^32；e=(t>>24)−126（t=0则−127），s=min(60,max(0,−e))。v=0或s≥48 返回1。否则 z=R32(v×2^s)，poly=(B⊖(z⊗A))⊕((z⊗z)⊗C)，返回(poly⊗4)⊕(R32(s)⊗R32(1/48))。使用本算法，不能用 meter F06L.log 或系统 log 替代。

## F06ND.4 primary、secondary 和 auxiliary peak

每 block、每通道 primary 当前peak=R32(2×max abs 256 samples)，combined=max(previous_primary,current)，然后 previous_primary=current。previous carry 跨帧。2是统计比例，不应用于主 PCM。

block0..4 对 secondary 的前五内部通道也做同一 current/previous 规则，并把 combined并入 corresponding primary combined；secondary的 LFE previous保持正零、不读取。block5没有 secondary读取。channel_peak=R32(max六个combined×1/128)。

每 block auxiliary 的输入采用 block0..4 secondary、block5 primary。设 scale=R32(V/64)，center=−R32(R32(N⊗V)⊗(1/32))，side同center，pl_side=−R32(R32(S⊗V)⊗(1/32))。逐 sample 先 l=L⊗scale，r=正零，c=C⊗center，l=l⊕c，r=r⊕c，r=r⊕(R⊗scale)。再按变体：

1. Lo/Ro：l⊕(Ls⊗side)，r⊕(Rs⊗side)。
2. Lt/Rt：先 term=Ls⊗side，l⊖term、r⊕term；再 term=Rs⊗side，同序更新。
3. rotation：l⊖((Ls⊗P0)⊗pl_side)，r⊕((Ls⊗P1)⊗pl_side)，l⊖((Rs⊗P1)⊗pl_side)，r⊕((Rs⊗P0)⊗pl_side)。

三个变体×左右共六行，每行取max abs，aux_current为六个最大值的max。LFE不进downmix。block0..4：aux_peak=max(aux_current,previous_downmix)，previous_downmix=aux_current。block5：scaled=terminal_peak_scale⊗aux_current，aux_peak=max(previous_downmix,scaled)，previous_downmix=scaled。peak_input[block]=max(channel_peak,aux_peak)。不可把 primary_only fallback 作为正式 domain。

验收投影字段 `auxiliary_raw_peaks_six` 明确对应每个block完成carry后的 `aux_peak`，即normalize之前、已经与 `previous_downmix` 取max的值；它不对应当块未携带历史的 `aux_current`。字段名中的raw只表示尚未normalize，不能据此跳过carry。

## F06ND.5 两种 limiter 和 none dynamics

scheduler先对六个raw peak计算 peak_log=clamp(normalize(peak_input)⊕(offset⊗1/2))。line按block0..5连续调用同一个 limiter；RF一次调用，输入 clamp(min六个peak_log⊖R32(11/288))。RF取min normalized log，不能换成max原始峰后猜域。

kind0=RF、kind1=line。更新 limiter(gain,hold)：v=peak_log⊖R32(7/48)，v≥R32(1/6)则v=1，否则v=v⊗6；target=v⊖R32(kind0:1/128，kind1:1/256)；incremented=gain⊕R32(kind0:3/2000，kind1:1/8000)。incremented≤target时，hold>0则gain不变、hold−1，否则gain=incremented、hold=0；incremented>target时gain=target、hold=kind0:5/kind1:30。等号走hold/release分支。

每 block none dynamics仍实际执行：level=1，hold=0，log_gain=min(0,limiter.gain)，按F32消费。虽然本空 profile 的结果不消费 RMS 的数值，producer仍计算和验证所有 RMS、peak、state；其它 profile不得借此省略。line每 block以本次 log_gain 量化 final/aux，并另量化 limiter.gain。RF先更新一次 limiter，然后六次 none dynamics；pre/post minima均从1开始，依次min(log_gain,min)，当前none两者相同；RF final/aux由该最小值量化，limiter code另由gain量化。禁止给none附加 profile-present 的第二次 gain clamp。

## F06ND.6 RF/line packed quantizer

输入 finite F32 g；kind0 multiplier1，kind1 multiplier2。q=Sat16(RoundEven(R32(R32(multiplier×g)×32768)))。按U16解释q，exponent=q&mask(kind0:0xF000，kind1:0xE000)，fraction=q&mask(0:0x0FFF，1:0x1FFF)。x=R32(R32(S16(fraction<<(kind0:3，kind1:2))×2^-15)−1)。p=R32(R32(R32(R32(x×x)×Q0)+R32(x×Q1))+Q2)，再p⊕bias(kind0:1/64，kind1:1/128)。

p≥1时比较**有符号**S16(exponent)与limit(kind0:0x7000，kind1:0x6000)：达到则p=1；否则exponent=(exponent+kind0:0x1000/kind1:0x2000) mod65536、p=0。r=Sat16(RoundEven(R32(p×32768)))；fractional_code=S16((r&0xFFFF)&kind0:0x3C00/kind1:0x3E00)算术右移kind0:2/kind1:1。combined=Sat16(S16(exponent)+S16(fractional_code))&0xFFFF。kind0且combined=0时返回U32 0xFFFFFF00，存储U16 0xFF00；其他返回combined。line的0不改成0xFF00。所有 `0x` 前缀常量均为十六进制，shift数量、kind、32768和65536均为十进制。frame_flag参数不影响这一个量化callee。

正常输出RF_flag=1；六个line_flag=1；minimum_line_signed=min(S16(line_codes))。state保存最后line dynamics、RF dynamics、两个limiter、六个peak_logs、generation+1。F12 high8转换严格是U16>>8。

## F06ND.7 事务、EOF、验收和未解决项

接口先检查必要参数已供应和身份、profile/config域、state类型及维度、generation一致、primary/secondary同代shape/finite、dialnorm域，再在owned临时状态计算。未供应报 `PARAMETER_UNAVAILABLE`；域外 `UNSUPPORTED_PROFILE`；形状/类型/代号报 `DRC_INPUT_ERROR`；中间非有限/溢出报 `NUMERIC_FAILURE`。失败无正常输出、不提交部分state；按I03退休并完整冷启动。该错误契约是独立实现要求，不声称原生域外行为相同。

正常EOF仅结束既有source extent的连续帧，不自行再补DRC专属zero frame、不单独flush biquad/limiter，也不把final true_peak_host塞入peak_inputs。上游frontend/history/codec定义其drain与截取，本接口消费其实际代数。最后仍核对scheduler.generation和overrideFIFO.frames_processed等于source_count；输出后不会reset到冷态。新的programme或明确reset才同时重置history、secondaryforward/inverse、statistics、scheduler、overrideFIFO及上游pipeline。

研究侧独立checker已验证31 offset words、3 mix words、12 normalized标量、7712 RF/line quantizer cases、186连续scheduler代（1116 line和1116 RF none dynamics）。全部消费输出和state逐词相符。另以独立构造primary/secondary运行8个连续统计帧、48blocks、240次fullband filter、30720个paired sample pair；全部统计投影及23个state words逐词相符。这组secondary是合成输入，未在本轮执行实际forward/inverse producer。

此次未新跑真实歌曲、未运行SDK、未读取DLL、未读取native PCM。既有OWN4249代控制文件的29743个成功flags已检查；RF唯一FF00、line唯一0是该输入的结果，不能推成全部音频结果。

13个功能值及其实际参数连续验收已供应。selected secondary producer、数值供参及本轮参数/向量的局部package-only审查均已通过；真实整曲F07/F12消费与whole执行仍开放。本规格不单独声明 `IMPLEMENTATION_READY=YES`；整包状态由所有必要模块和验收门决定。private key不参与上述算法或参数。

局部参数化文档r2独立复审通过，34个数学生成词、19项合成检查与3项问题关闭检查通过；三项文档问题均关闭。见 `test-vectors/NONE-DRC-PARAMETERIZED-LOCAL-REVIEW-STATUS.json`。该r2审查只对应原参数化材料，不自动覆盖本轮新实际参数/向量或secondary完整供应；新参数供参与连续向量已由NONE-DRC-ACTUAL-LOCAL-REVIEW-STATUS.json独立审查通过。

## F06ND.8 授权实际功能值与 input-only 连续验收

`parameters/none-drc-authorized-v1.json` 的四组顺序为 FILTER5、NORMALIZE3、QUANTIZE3、AUX_ROTATION2；共13个U32位模式，按该顺序little-endian序列化52字节。U32是权威binary32身份，decimal_annotations仅便于阅读，不能重新舍入十进制注释。前5个值绑定既有历史DEE非秘密静态音频证据；后8个值绑定已核验冻结Python模型的functional consumer，其原始编译位置未在本轮新确认。不会把这些值宣称独立数学生成设计，来源分类保留。仅内部交付和实现，原表文件、源码、DLL、private key/认证材料均不交付，不授权公开或第三方分发，也不扩展其他依赖的权限。

数学34个词由 `parameters/none-drc-math-v1.json` 提供；本轮 runtime 参数只消费授权13词及这34数学词，不用原125项offset表或3张8项mix表。完整实际参数网络没有近似替代；局部等价范围是下列输入、状态及该选定profile。

`test-vectors/NONE-DRC-AUTHORIZED-VERIFICATION.json` 包含8个统计连续帧、186个scheduler连续帧、12个normalized独立输入和7712个RF/line quantizer独立输入。所有配方只依赖input（或input_recipe_index）与实际参数。statistics_sequence和scheduler_sequence是两条独立序列，各只在step0前冷启动，之后只能消费前一计算产生的实际next_state；禁止拿expected_next_state作种子。统计state是23个F32字，完整记录filter5×2、primary_peak6、secondary_peak6及downmix_peak1；scheduler完整state记录line/rf dynamics、line/rf limiter、peak六项及generation，浮点位模式和整数hold/generation严格区分。

统计每帧独立配方构造primary/secondary各6×1536 finite F32，顺序L,R,C,LFE,Ls,Rs；secondary仅此合成测试输入，不能宣称新真实forward/inverse重构已验收。实际参数测试覆盖正零、负零、跨block脉冲、幅度大于1的截幅、静音历史延续、交替信号和DC；全部output投影与23 state词逐词精确。两条主连续序列分别全程固定programme dialnorm=24，不在同programme自动换dialnorm；offset数学供参另完整验证全部31个d。scheduler覆盖静音/小峰/递增峰/高峰/交替峰以及limiter持续状态。normalized和quantizer每例独立、无跨例状态。

结果 `test-vectors/NONE-DRC-AUTHORIZED-RESULT.json`：48统计blocks、240fullband filter calls、30720paired sample pairs，line/rf各1116 none dynamics calls，12 normalized与7712quantizer，全部声明的输出和state精确、first_divergence=null。检查同时绑定参数JSON、13词52字节与vector JSON的SHA256。新root侧重跑工具同样仅从clean参数和input配方重算，expected仅在计算后比较。未运行SDK、未读取DLL、未用native PCM/private key、未新跑真实programme，实际secondary transform供应仍须独立闭合。
