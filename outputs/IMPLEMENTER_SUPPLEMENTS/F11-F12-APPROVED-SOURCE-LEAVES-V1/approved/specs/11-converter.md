# F11 虚拟转换器预算

状态：`SELECTED_PUBLIC_BAP_LOCAL_REVIEW_PASS_WHOLE_INTEGRATION_OPEN`。转换策略、成本、搜索和selected D13公共BAP provider已交付并通过局部r2复审；完整codec消费集成仍为Q09。

## F11.1 独立输入

converter 使用保存的 POWER 刷新后指数行重新运行 F09.3 的普通刷新，再按 F09.4 选策略并匹配 frame pattern。它不消费主 frame code。

快照 S 明确是主 POWER 的第一次刷新返回的 current 行，形状为每通道六行、每行 217 项 I16。在该块首次刷新结束时冻结：若该分支限制了 current，则 S 含限制后的值；结构刷新只限制 work 时，S 保留原 current。索引 128～216 保持 raw 结果。S 既不是 raw 的无条件副本，也不是 work，更不是后来写入的 PRIMARY 行。

converter 为本通道每帧新建独立的 217 项零 work，使用 S、同一组 consumed short/other 所派生的结构 markers、mode1/secondary0、区间 [0,128) 重新刷新，保存 converter 自己的 current/work/markers。short 覆盖及反向阈值按 F09.4，随后做独立 frame pattern 匹配。不得引用主刷新最后的 work，也不得对 PRIMARY 再做 POWER 刷新。

序列化后的主解码指数按 converter 的复用区间再次逐 bin 取最小，并执行 option=0 的差分限制和 span 展开；全带宽长度 217、LFE 长度 7。conversion 不使用主 optimizer 的 option1 第二轮。全带宽区间边界由converter策略决定。

LFE的converter clock逐块直接取主LFE plan的strategy（六个0/1）；converter不另跑LFE POWER刷新，也不引用FBW frame pattern。输入LFE六行来自主LFE的decoded_exponents_for_BAP，再按该clock做相同的区间min/option0 conversion。首clock必须1，reuse继承前anchor；LFE无gain字段。

内部虚拟预算固定为 20480 bits（640 kbps），不改变实际 32768-bit、1024-kbps EC3 输出。converter fast gain code=4；主 fast gain code=1。两者的 decay tuple 都为 slow_decay=2、fast_decay=1、slow_gain=1、当前块 db code、floor=7。

候选 q 的 coarse=q>>4、fine=q&15，q=0 时返回全部零 BAP。q>0 的 BAP 输入为该块转换后的指数、上述 tuple、fast gain=4、起 bin=0、无 coupling/delta allocation。完整参数化数值网络见F12A；D13 selected公共BAP表已交付并通过局部r2复审。

## F11.2 固定成本

当前 acmod=7、一个 programme，虚拟 header 包括 RF compression、dialnorm、混合信息和可选 surround EX 信息；成本为 `91+14×int(surround_EX>0)` bits。该式仅用于固定字段组合；任何 profile 变化重新提炼成本。

令块 b 五路 converter 策略为 s、相应 packed group count 为 g，LFE refresh 为 r。当前 line compression 字段存在，故基础成本从 10 开始，加 9、首块 2/其余块 1、五路策略和 LFE 策略 11、4 个固定 bits；每个非零全带宽策略再加 `12+7g`，LFE refresh 再加 18。

BA record update 为首块 1，后续为当前 db code 与前块不同；为 1 时加 11。SNR record update 仅首块为 1，此时加 48。所有成本步骤按 W16；结果必须非负。auxiliary absent 成本为 1。EMDF skip 不等同于此 auxiliary。

Mantissa histogram 每块覆盖五路 217 bins 和 LFE 7 bins，按 BAP 0～15 分类。码 1、2 的数量分别改为 ceil(count/3)，码 4 改为 ceil(count/2)，其他不变。各类每组/每项位宽为 `[0,5,7,3,7,4,5,6,7,8,9,10,11,12,14,16]`；sum(count×width) 按 W16，结果必须非负。跨通道分组，但每块重新开始。

## F11.3 可行性

记 frame=20480、header=H、六块固定成本 F[b]、mantissa 成本 M[b]、aux=A。初始剩余 `R=W16(frame-H-sum(F)-sum(M)-A-17)`；R<0 时返回成本失败。

首分区容量为 12800，尾容量为 7680；逐块 quota=3413。以累计容量 0、已花费 H、剩余 R 开始，按块 0～5 执行。先累计容量 W16 加 quota，已花费 W16 加 F[b]+M[b]。available 为 `min(W16(累计容量-已花费),剩余)`。

available>=9 时，stuff groups=`min(floor((available-9)/8),511)`，stuff cost=`W16(8×groups+9)`；否则 cost=0。已花费加 cost，剩余减 cost，各用 W16。

首分区剩余为 `W16(12800-stuff[0]-stuff[1]-M[0]-M[1]-H-F[0]-F[1])`。尾剩余为 `W16(7680-remaining-A-M[5]-17)`。返回残差取初始 R、首分区、尾分区中的最小值；任一严格更小且负值时不可行。注意尾计算不减 F[5]，不能自行补项。

## F11.4 SNR 搜索

区间 low=0、high=1024、best=-1。每步 q=floor((low+high)/2)，包括 q>671 时仍计算候选 BAP 和成本，然后判断 q<=671 且残差>=0 才可行。

总预算先减 H、六块 F、A 和 17，再逐块减 M[b]；中途负值则直接失败，不进入分区检查。其他情况按 F11.3 检查。可行则 best=q、low=q；不可行则 high=q。high-low<2 时终止，边界相等视为可行。

若 best 仍为 -1，用六块零 mantissa 成本做 F11.3；可行则结果 0，否则返回失败。成功 offset 为 10 位 U16 值，当前 cap=671。试算不推进任何音频、随机、元数据或认证状态。
