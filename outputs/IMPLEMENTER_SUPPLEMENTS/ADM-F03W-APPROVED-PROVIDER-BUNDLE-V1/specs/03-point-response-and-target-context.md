# F03P Point response 与 target context

本文件定义当前 selector/outer 实际消费的 15 点 response。音频 prefix 仍按 F04G 使用原始 width；此处消费音频路由后的独立 feature-source 视图。数学角函数为新的完整候选路线，真实 ADM 新路线的音频/空间验收仍未完成。

## F03P.1 输入视图与实际消费集合

输入为全局 source 0～47 的语义记录：XYZ 三项 finite F32、original_audio_width F32、feature_mode U32、source_skip U32、spatial_enum U32、coordinate_parameter U32。XYZ 来自 F01B/sampler；skip 是路由源的原始控制字段，不是 F03R 的 processing_inactive。

所有 source 的当前 feature_mode 必须为 0，否则拒绝。音频路由完成后，为每条 source 创建 feature_width=正零的视图。保留原始 audio width 和源记录，不在此修改 F04G 的输入。该转换属于当前已恢复 route 分支；备用 mode/config 的 writer 行为仍未解决。

需要 response 的 source 是 F03G.1 的 candidate 与 solver-call 两个集合的并集，source 索引递增且去重。只为这些 source 生成列。每个实际生成 source 要求 skip=0、feature_width=0、spatial_enum=0、coordinate_parameter∈{0,1}；不满足时拒绝，不供应 native/参考列。

生成输出为 source ID→15 项 F32 的完整列。15×48 的诊断展开按 basis-row major：未消费单元为 absent/null，不能声称为数值零。F03S 的最终 n×48 系数矩阵则将未调用 source 写零，两者不是同一矩阵。

同时输出每个生成source的feature_width位模式。F03C.1的width[j]按candidate source身份读取这里的post-audio feature_width，并将位模式解释为F32数值；当前为正零。不得改用original_audio_width，或因两者都叫width而选择任一字段。F03C的独立参数化用例可以供应非零width，不代表当前选定路径会供应非零值。

## F03P.2 15 点 basis 与量化

basis[15][3] 为 finite F32，按 provider 的稳定顺序。D10-P15 的实际供应是 D10-G/D10-LX，经 F01B.2 的单步 fallback 和 profile high-Z X 调整，但 layout=0x159B、profile bit35 启用，并要求恰好 15 个点。不得用 11 点 prefix 的 layout=0x111B 或歌曲 metadata 代替。

XYZ 按 F03.2 的 Q15 规则：先乘 32768 并 F32 舍入，再加符号对应的 ±0.5 并 F32 舍入，向零截断，正端上限 32767。输入/中间结果须 finite，最终整数须能存入 I32；没有负端 clamp。随后各坐标除 32768 并转 F32。

coordinate_parameter=0 时，在完成量化后把 Z 整数改为 0；X/Y 保持。它不是整列的 disabled 开关。parameter=1 保留量化 Z。

分组按 F04G.3：lower 的 -0.1<z<=0.5、upper 的 z>0.5，稳定锚点 y 距离比较、组内 x 和组间 y 稳定排序。本模块要求各 basis 点都属于一个平面，两个平面均非空。几何 provider 缺失或无效时失败。

## F03P.3 Point 平面与高度组合

每平面按 F04G.3 生成 x 组内权重和 y 组间权重，逐点 plane_weight=x_weight⊗y_weight。这里的 EQP 使用 F03P.4 的 angle_mode，不使用 prefix 的固定数学函数配置冒充旧 FMA3。

量化 Z<=0 时 height_low/high=1/0；floor(Z⊗32768)>=32767 时为 0/1；其余为 EQP(Z)。对每个 basis 点 j，response_j=(lower_plane_j⊗height_low)⊕(upper_plane_j⊗height_high)。各平面数组没有该点时为正零。

最终 threshold=R32(10^-6)，已与当前消费标量逐位核对。仅 threshold>response_j 时清正零；相等保留。没有额外归一化。negative/nonfinite 中间值按本条款与 N00 检查，不用另一条 spread 网络补输出。

## F03P.4 数学角函数候选路线

当前隔离候选 angle_mode=`MATHEMATICAL_R64_R32`。对 r∈[0,1] 的 finite F32：r<=0.5 时 a=((r⊗R32(pi/4))⊗0.5)⊗4，输出 (M32(cos(a)),M32(sin(a)))；另一侧以 1⊖r 构角并交换两项。M32(f)=R32(R64(f 的精确实数结果))，禁止把两次舍入合并为一次 R32。

角域为 ±零或 0<a<=R32(pi/4)。零时 sin 保留输入零符号，cos 为正 1。以下完整有理数区间规则可取得 M32，生产不依赖原编译 polynomial 表：

将输入 F32 的精确数值表示为有理数 x。sin 初始 term=total=x，cos 初始 term=total=1。迭代 index k 从 0 开始：sin 的 next_term=−term×x²/((2k+2)(2k+3))，cos 为 −term×x²/((2k+1)(2k+2))。所有这里的运算都是精确有理数，不是 F32/F64。

next_total=total+next_term。由该角域下的交错级数余项界，真实结果在 total 与 next_total 之间。两端分别按 N00 的最近偶数 R64 舍入；若二者相同，已唯一确定 R64 结果，再执行 R32 并结束。否则保存 next_term/next_total，k 加 1。不得以 fixed decimal digits 或误差阈值代替这个停止条件。

实现可采用满足同一结果定义的已核验数学库或更快算法，但验收以区间参考结果为准。参数描述见 `../parameters/point-panner-math-v1.json`。局部库比较不构成全输入正确舍入证明，不能将任何系统 libm 名称当作天然合格。

## F03P.5 原 FMA3 参数化路线

可选 `FMA3_PARAMETERIZED` 保留原精度路线。它要求 D10-A：S[4]、C[4] 与 quarter_pi、tiny_cutoff、small_cutoff、one_sixth、two_over_pi、half_pi_hi、half_pi_tail 共七项 F64。值未交付时该模式报 PARAMETER_UNAVAILABLE；不能改填数学候选值后宣称旧 FMA3 精度已复现。

下面每个普通运算分别 R64；只有 FMA64 融合。对角 x：x<tiny 时 sin 返回原 F32 位模式、cos=1；x<small 时 sin=R32(FMA64(−((x×x)×x),one_sixth,x))，cos=R32(FMA64(−(x×0.5),x,1))。

定义 sine polynomial：t=x×x，v=FMA64(t,S3,S2)，v=FMA64(v,t,S1)，v=FMA64(v,t,S0)，结果=FMA64(v,x×t,x)。定义 cosine polynomial：t=x×x，普通 base=1−(t×0.5)；v=FMA64(t,C3,C2)，v=FMA64(v,t,C1)，v=FMA64(v,t,C0)，结果=FMA64(v,t×t,base)。

x<=quarter_pi 时直接取两个 polynomial 并 R32。仅支持角域剩余 endpoint 的 reduction：m=trunc(FMA64(two_over_pi,x,0.5))，要求 m=1；h=FMA64(−m,half_pi_hi,x)，u=h−(m×half_pi_tail)。sin 用 u 的 cosine polynomial，但 base 改为 FMA64(u²,−0.5,1)；cos 为负的 u sine polynomial，再 R32。原 reduction compensation 的诊断值不参与结果，不能误加到 u。

该模式与数学候选模式分别验收，不在运行中按误差或 source ID 混用。原 FMA3 库与数学候选的有界差异见局部报告；整链候选质量仍为 NOT_RUN。

## F03P.6 Installed target 到 context

F03G 安装后、F05H marker 修改前，先构造全 15 个 fit-control 记录，见 F05H.1。当前 context 路线要求每条 fit 的 mode=0、width=0、extra=0、spatial_enum=0、coordinate_parameter=1；quiet/inactive_pair 与前面的 attribute 分类字段不参与 point 数值计算。

fit-control的适配固定如下，词编号从0开始。它不是PREFIX_CONTROL_INPUT32_LE的隐式复用；各字段由本表独立绑定。

| fit word | 位类型 | context语义 |
|---:|---|---|
| 6/7/8 | F32原位模式 | XYZ |
| 9 | U32 | mode |
| 10 | F32原位模式 | width |
| 11 | U32 | extra |
| 12 | U32 | spatial_enum |
| 13 | U32 | coordinate_parameter |
| 0～5 | F05H原类型 | 此point/context数值不消费 |

word10的构造U32正零按F32解释为正零；不能把任意U32数值直接转换为浮点width。当前F05H从attribute构造word9=A AND1、word12=(A>>1) AND7，因此这些位非零会在context合法性校验失败，即使该slot inactive也不能忽略。这里的coordinate_parameter不是prefix记录中同位置名称enabled的推定别名。

为全 15 个 capacity slot 按 F03P.2～5 算出 15 项 point response，再施加同一 threshold，得到 full_context[15 basis][15 slots]。按 basis 索引递增、active destination slot 递增投影为 context[15][n]，n=安装后 active 槽数。不是按本次 selected_count 取前 n 列；空选择可能保留旧 active。

n=0 时投影结果为空，仍检查全 15 个 fit 的合法性。当前已接受 consumer 先构造完整 context，再取 active 列；不能因 inactive 而接受未恢复的 fit 分支。outer 输入 source/target XYZ 取同 callback 的几何/安装快照，不取下一代 marker/history XYZ。

## F03P.7 状态、失败与验收

basis/angle_mode 在 programme 初始化时安装并冻结。每 callback 的 feature-source view、response columns 和 context 为局部结果，不携带前 callback 数值。对象原始 width、target state、feature history 和 marker/history 各按所属条款拥有；本文件不把它们清零或提交两次。

类型、尺寸、source集合、quantizer范围、参数或数值错误执行 I03 fail-stop。当前不证明全部 native 中途失败提交。生成失败时不以 native/cache/零列补齐。

`../test-vectors/POINT-RESPONSE-LOCAL-VERIFICATION.json` 核验角落、中心、高度参数0、半高度、post-audio width视图/selector绑定、未消费null、active context、attribute构造fit及拒绝、组内不同y的首点改变。数学区间与库、旧FMA3的4112次角值比较覆盖2056个角；5项与旧FMA3不同，最大1 ULP/5.960464477539063e-8。max_next_term_iterations统计next_term计算次数，不含初始term；最多10次，即含初始term最多11项。生产basis、新路线真实ADM和完整空间质量尚未核验，局部初审的问题修补须复核。
