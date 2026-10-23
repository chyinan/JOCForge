# F06L Meter 的 log、energy metric 和 peak

本文件定义 meter 的局部标量和 normalized energy 网络。K-weighting、窗口调度、integrated gate、true-peak FIR 和完整控制通路仍有独立缺口。仅限内部研究和内部实现。

## F06L.1 参数和数值规则

参数按顺序为 R32(1/7)、R32(1/8)、R32(1/6)、R32(1/5)、R32(1/4)、R32(1/3)、R32(1/2)、R32(ln(2)/2^31)、R32(1/ln(10))、R32(2/ln(10))、R32(7ln(2)/(32ln(10)))，编号 C0～C10。R32 从精确数学值最近偶数舍入到 binary32。生成规则和固定词见 parameters/meter-log-mathematical-v1.json；原研究表未复制。

以下 ⊕、⊖、⊗ 按 N00 每步分别舍入到 F32，无 FMA。输入标量须有限，offset 为有符号整数。中间结果非有限或指数构造超出范围时，按 I03 失败，不能宣称该输入已验证。定义 P(k) 为 normal binary32 的精确 2^k，要求 −126≤k≤127。

shift(v)：先取 R32(v) 的 U32 位模式 u；t=(2u) mod 2^32。t=0 时 e=−127，否则 e=(t>>24)−126。返回 min(60,max(0,−e))。这是位模式规则，不能换成平台 frexp 后自行处理 subnormal。

## F06L.2 有序 log 网络

log(v,o) 先将 v 转 F32。v≤0 返回 −1。正值令 s=shift(v)。s≥31 时令 r=o−s；r<−46 返回 −1，否则返回 (R32(r×2^25)⊗C7)⊗2。

s<31 时：x=(P(s)⊗v)⊖1；p=C0⊖(C1⊗x)。按 C2、C3、C4、C5、C6 的顺序更新 p=Cj⊖(p⊗x)。然后 l=x⊖((p⊗x)⊗x)，a=R32((o−s)×2^26)⊗C7；结果为 (l⊗(1/32))⊕a。

必须保留该多项式和分支，不能用系统 log 或数学 ln 替换。参数生成中的 ln 与运行时的 log(v,o) 是不同操作。

## F06L.3 Energy metric

energy_metric(m,o) 要求 m≥0，先转 F32。令 s=shift(m)。m=0 或 o−s≤−39 时返回 −15/16。

若 o−s<−32，n=P(s)⊗m，l=log(n,o−s+8)，a=(l⊗C8)⊖C10。否则 l=log(m,o+1)，a=l⊗C8。返回 (2⊗a)⊕((1/2)⊗a)。−39 的等号返回 floor，−32 的等号走后一分支。

这是 normalized mantissa/exponent 的指标换算。权重、归约、窗口采样数和 integrated gate 不由本条款补出。

## F06L.4 Sample peak

输入至少一行，每行恰好 1024 个 finite F32 样本。各行独立计算：peak=abs(row[0])、index=0；按索引 1～1023 扫描，仅 candidate>peak 时替换。因此相等峰保留第一次位置，全零行 index=0。

peak<R32(10^-6) 时 internal=−15/16。否则 v=min(1,max(−1,peak))，internal=(log(v,0)⊗C9)⊗(5/2)，再先 min(+15/16)、后 max(−15/16)。host_metric=internal⊗128。输出 maximum_abs_sample、first_maximum_index、internal_log_metric 和 host_metric；没有跨行 peak state。

## F06L.5 True-peak 指标

本接口消费 FIR 已产生的非负 finite raw_peak，不提供 FIR。先转 F32。raw_peak<R32(10^-6) 时 internal=−15/16；raw_peak>R32(10^6) 时 internal=+15/16；其余 internal=(log(raw_peak⊗(1/4),2)⊗C9)⊗(5/2)。host_metric=internal⊗128。

中间区间没有额外 clamp。不得把 >10^6 改成 ≥10^6，也不得借最终 floor/cap 掩盖中间非有限值。当前局部验收覆盖普通幅度、低阈值相等/下邻和大于上阈值的直接 cap；恰好上阈值及该大幅度多项式的完整有限域没有证明，仍是数值边界缺口。

## F06L.6 验收范围

METER-LOG-GENERATION-VERIFICATION.json 使用独立配方构造输入。log 与 energy 的 expected/actual 是输出 F32 的十进制 U32 位模式。sample_peak 从 1024 个正零开始，在 negative_position 写负 peak、positive_equal_position 写等幅正 peak；peak=0 保留正零。四项投影为 maximum_abs_sample_word（maximum_abs_sample 的 U32 位模式）、first_index（first_maximum_index）、internal_word（internal_log_metric 位模式）和 host_word（host_metric 位模式）。peak=2 的 maximum_abs_sample 保持 2，指标的 clamp 不修改该峰值。true_peak 直接消费 raw_peak。每例独立，不把 expected 当作输入。

100/200 位生成结果一致，11 个常量词身份吻合；56 个局部案例逐词一致，包含下文 20 个 energy 原语案例。验收范围是这组固定参数和列出的输入，局部无研究上下文审查已完成；完整meter、真实ADM、新编码及全链验收仍开放。本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。

## F06L.7 Normalized energy 累计

接口消费旧 mantissa m、整数 exponent e 和当前非负 power_sum p。m、p 先按 F32 消费，必须有限且非负；所有 P(k) 遵守 F06L.1 的 normal 范围。power_sum 的 K-weighted 求和 producer 和窗口重置时刻仍未提供，不能从本原语推断完整 meter 时钟。

令 s=shift(p)。若 e≤s，result=(P(e−1)⊗p)⊕((1/2)⊗m)，next_e=e−1。否则 d=e−s+1，d≤31 时 retained=P(−d)⊗m，d>31 时 retained=正零；result=(P(s−1)⊗p)⊕retained。result=0 时 next_e=e，否则 next_e=s−1。返回 result 和 next_e，不提前量化成普通单个能量浮点数。

## F06L.8 窗口末归一化

此原语可独立调用。输入 m 转为 F32 后必须有限且非负；两种零均允许，保留符号位。e 必须为有符号 I32 整数，window_samples n 为整数且 1≤n≤48000。类型、转换溢出、非有限或负 m 在运算前拒绝；所构造 P(k) 超出 normal 范围时拒绝，不产生可用结果。q=(R32(2^31)/R32(n))⊗R32(2^-31)，其中除法本身也舍入到 F32。这两个固定值分别等于本路线的大整数转换和倒数常量，不直接改为 R32(1/n)。

令 s=shift(q)，out=(P(s)⊗q)⊗m，out_e=s+e−16。out_e<0 时先 out=P(−out_e)⊗out，再将 out_e 设为 0。返回 mantissa=out、exponent=out_e。调用后是否清累计 state、窗口多长及 EOF 残余块的 n，由尚未闭合的窗口调度定义。

## F06L.9 有序加权归约

输入 exponents、weights、energies 三个等长数组，长度 0～16。每项 source_exponent 为整数 0～60，weight 和 energy 为非负 finite F32。按数组顺序处理，不能排序或并行归约。acc 从正零开始，e=48。

每项先 weighted=weight⊗energy。e≤source_exponent 时 d=min(30,source_exponent−e)，然后 e=e−1、added=P(−d−1)⊗weighted、previous_scale=1/2。否则 d=min(30,e−source_exponent)，然后 e=source_exponent−1、added=weighted⊗(1/2)、previous_scale=P(−d−1)。两种分支都更新 acc=(previous_scale⊗acc)⊕added；d 使用本项更新前的 e。

全部项处理后，若 e>27，acc=acc⊗P(−(e−27))，再令 e=27。然后 s=min(30−e,shift((1/8)⊗acc))，返回 mantissa=P(s−3)⊗acc、exponent=s+e。空数组也执行该末尾归一化，不跳过或返回未定义字段。

accumulator、window 和 reduction 案例的 expected/actual 均为二元数组 [mantissa 的 U32 位模式, exponent 整数]；每例仅使用其 input，独立执行一个原语。生产窗口持续累计、gate、失败提交及完整 meter terminal 尚未验收。

rejection_cases 只从 input 构造独立 window 请求，expected_rejected/actual_rejected 表示入口拒绝；覆盖负 m、非整数 e 和转 F32 溢出。拒绝后没有可用结果。它们核验本包的输入域，不声称原生在域外同样拒绝。负零 m 的正常案例保留 80000000 的输出 mantissa，不能因非负域检查清成正零。
