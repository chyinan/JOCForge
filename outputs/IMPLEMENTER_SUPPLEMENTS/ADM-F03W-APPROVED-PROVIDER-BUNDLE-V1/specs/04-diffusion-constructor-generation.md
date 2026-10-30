# F04FCM 固定 48 kHz diffusion 构造配置生成

本条款给 D11-DP 提供完整数学参数化。它与 F04FM 的系数/配对规则一起构造 F04F 冷状态。范围固定 48000 Hz；参数值的等值重建不等于已证明原设计采用了这些工程参数或公式。交付仅供内部研究和内部实现，不公开或向第三方分发。

## F04FCM.1 十项配置

以下各结果按最近偶数转 F32。顺序与 F04F.1 的接口一致。

| 参数 | 生成规则与单位 |
|---|---|
| ring_seconds | 1/500 秒 |
| bank_total_seconds | 1/50 秒 |
| bank_geometric_base | 3 |
| bank_exponent_step | 1/4 |
| gain_seconds | 1/20 秒 |
| slow_seconds | 2/25 秒 |
| fast_seconds | 1/200 秒 |
| input_seconds | 下一节最近偶数整数纳秒除以10^9 |
| envelope_epsilon | 1/100000 |
| envelope_limit | 11/10 |

这些是恢复行为的等值生成设置，不把它们宣称为新的公共协议常量。原研究配置表或源代码不作为运行依赖，不随包直接转交。

## F04FCM.2 输入时间参数的唯一生成

生成参数名义极点频率 f=48000/16=3000 Hz。定义 T=1/(2πf) 秒，n=RNE_integer(10^9T)，input_seconds=R32(n/10^9)。这里名义频率仅用于等值参数化，不声称它是该完整检测网络实际的−3 dB截止频率。

用 Machin 恒等式 π=16atan(1/5)−4atan(1/239) 生成有理数区间。对 atan(x)，初始 term=sum=x，k=0；next_term=−term×x²×(2k+1)/(2k+3)，next_sum=sum+next_term。运算为精确有理数，相邻两个 sum 给出上下界。分别取得 atan 两项的 lo/hi，组合 π_lo=16lo_5−4hi_239、π_hi=16hi_5−4lo_239。

将纳秒区间设为 [10^9/(2fπ_hi),10^9/(2fπ_lo)]。仅当两个端点按最近偶数取整得到同一个整数才停止，否则继续迭代。最终 n=53052；再除10^9并R32，不用原始时间常数值作为输入。

## F04FCM.3 固定构造结果

依 F04F.1 的独立F32乘加、pow64/exp64、ceil和trunc规则生成。bank ratios按s=0～3取R32(R64(3^(s/4)))，分母按s递增F32累加。得到四个 bank 长度 [152,200,263,346]，ring长度96。

gain/slow/fast/input smoothing各按 C(t)=R32(R64(exp(R32(−1/(R32(48000)⊗t)))))。这里除法是F32，exp结果先R64再R32。固定输入的100/200位精度复算结果一致；这不构成任意数学库、任意输入的正确舍入证明。运行时可读取给定生成数据，不依赖平台libm恰好重建同字。

数据为 `../parameters/diffusion-constructor-mathematical-v1.json`，包含十项配置词、四个bank ratio、四个smoothing输入/结果、长度及字段顺序。F04F.1定义完整冷状态：两项gain为1，所有bank/ring/accumulator为正零，索引全0，fade_pending=true。F04FM提供coefficient，不能把本配置数据当作缺省零系数。

## F04FCM.4 等价与边界

核验见 `../test-vectors/DIFFUSION-CONSTRUCTOR-GENERATION-VERIFICATION.json`。十项有效F32配置与选定消费参数逐项相同，完整冷状态逐字段相同。4次连续处理使用显式合成PCM规则，所有四路输出和完整next_state相同。

声明的等价仅为固定48k构造器及配置替换：初始状态和全部有效参数相同，F04F中同一运算/分支序列、同输入时状态转移相同。没有通过近似容限判PASS，也没有用未知值补全参数。其他采样率、layout、上游controller和真实ADM整链不由此推定。

## F04FCM.5 安装与失败

配置在programme初始化时生成或读取固定数据，校验完整字段、类型、数量、sha与合法域。reset重新构造全状态；不能只清包络而保留旧bank/ring。生成越界、数据缺失、身份不符或非finite按I03停止，不回退到研究表。

本次原研究配置未复制，内部规则型交付不授权公开或第三方分发。独立局部复审已通过生成值、输出及规范状态摘要，见DIFFUSION-COMPLETE-LOCAL-REVIEW-STATUS.json；原研究参数来源对照未由隔离审查者重做。本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
