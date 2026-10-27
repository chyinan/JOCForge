# F10 指数规范化与分组

状态：`SELECTED_POLICY_PARAMETERS_SUPPLIED`。本文件定义全带宽217 bins与LFE7 bins的参数化分组规则；生产normalization_factor D06-N由 `parameters/policy-authorized-v1.json` 供应。

## F10.1 复用区间

一帧内每个非零策略开始一个区间，后续连续零策略属于该 anchor，直到下一个非零策略或帧末。每项 PRIMARY raw 指数在整个区间取最小；gain range 也在整个区间取最小。先完成六块的区间合并，再规范化 anchor，不能边到块边序列化尚未合并的指数。

输入 raw 行为 I16，范围 0～24；每通道 6 行，全带宽行长 217，LFE 行长 7。首块必须非零策略。gain range 为 U2，LFE 不提供 gain range。

## F10.2 分组准备

设行长为 n，span 为 s，全带宽 s∈{1,2,4}，LFE 固定 s=1。基础项为行第 0 项。剩余 n-1 项按 s 分组，组数 `g=ceil((n-1)/s)`；不完整组用最后一项填满。

每组取 `min(24,组内各项)` 得到代表值，把该代表复制 s 次形成初始分组行 q。option=1 时保存 q 的全长副本，初始和按元素递增顺序 W32 累加。先保存再规范化，不保留未分组 raw 作为误差基准。

## F10.3 差分限制

base 取 `min(raw[0],15)`。按组序计算代表值相对上一代表的 W16 差分；第一项相对 base。对差分序列做确定的修正：

- 正差分 >2：当前项 W16 减 1；若有下一项，下一项 W16 加 1；继续检查同项。
- 第一轮负差分 <-2：有前项时，前项 W16 减 1、当前项 W16 加 1并退回前项；无前项时，base W16 减 1、首差分 W16 加 1。
- 已在允许区间的项：前进到下一项。

扫描到末尾终止。累积 base 和差分恢复代表值，再将代表值复制 s 次。处理后要求 base∈[0,15]、所有指数∈[0,24]，否则失败。

第二轮仍对正差分 >2 使用同一修正。负差分的接受条件改为 `delta>=-1 OR group_offset>=20`，其中 `group_offset=组下标×s`，下标针对去掉基础项后的分组行。其余负差分修正规则相同。

## F10.4 option 1

第一轮展开行 p 与初始 q 比较：按元素递增顺序计算 `E=W32(E+W32((p[i]-q[i])^2))`。令 `c=W16(sg)`。

必要参数normalization_factor为1个有限非负F32，以授权provider的U32位模式供应。gate非零时factor=F32正零；gate为零时factor取该参数。`u=R32(c)⊗factor`。u<1时，门限`T=rne_int(max(-2147483648,u⊗2147483648))`；u>=1时T=2147483647。2^31是本数值转换的精确F32界限。

当且仅当 `W32(E<<16)<T` 且初始和严格小于 `W32(16c)` 时触发第二轮。重新对第一轮已经规范化的行取组内最小值，然后按 F10.3 第二轮规则处理。不能从旧 q 或 raw 重新开始。

option=0 跳过误差比较和第二轮。当前主 profile 为 option=1；LFE 的 option 同样为 1，但 gate 独立定义。

实现必须核验授权provider版本、selected_profile_id、字段顺序与位模式。option1/gate0缺参数时返回PARAMETER_UNAVAILABLE；不用合成factor继续生产。旧合成单元向量中的自定义参数只检验操作网络，实际参数向量见F09PA。

## F10.5 序列化

最终各代表的差分范围为 -2～2；digit=delta+2，范围为 0～4。每三个 digit 用 `25d0+5d1+d2` 编为一个 7 位无符号值，范围 0～124。不足三个 digit 的尾组以 digit=2 填充。

基础指数宽 4 位；全带宽 gain range 宽 2 位。reuse 块的基础指数、packed groups 和 gain 字段省略；BAP 仍消费 anchor 的完整展开行及继承后的 gain 值。

全带宽 s=1/2/4 的组代表数为 216/108/54，packed 数为 72/36/18。LFE 有 6 个差分、2 个 packed group，始终 s=1。LFE 每块分别计算 BAP，不能只使用帧首或末的 BAP。

## F10.6 音频计划与失败连接

F09 的 POWER 刷新策略与 PRIMARY raw 序列化指数是独立输入。全带宽先完成该帧全部 reuse 区间的 PRIMARY/gain 最小值合并，然后各 anchor 运行一次本算法。每个 reuse 块获得 anchor 的完整展开指数和继承的 gain，不能用歌曲策略缓存代替计算。

LFE 的六个 marker 直接作 1/0 策略，首块必须刷新。在其各 reuse 区间，7 项 PRIMARY raw 按 bin 取最小值，再调用 span=1 的 normalization。option 和 LFE 独立 gate 显式传入；当前选定值为option=1、gate=0。gate由F09LB的selected producer产生，不能从全带宽gate推导；AHT-enabled扩展profile不在当前支持域。

正常输入raw为六行严格整数0～24。全带宽strategy为六个严格整数0～3，首项非零，anchor的1/2/3分别对应span=1/2/4。LFE strategy只接受六个严格整数0或1，首项必须为1，各anchor固定span=1；LFE输入2或3报 `EXPONENT_INPUT_ERROR`。全带宽gain为六个严格整数0～3。bool不算整数。option只接受0/1；gate是显式I16。D06-N必须来自授权provider，不能用有理候选或字面值绕过身份核验。

option=1 且 gate=0 缺 D06-N 时返回 `PARAMETER_UNAVAILABLE`。option=0 不消费该参数；gate 非零使用本算法明确定义的正零 factor，不是为 gate=0 填占位参数。输入或输出范围不符返回 `EXPONENT_INPUT_ERROR`；非有限中间值或数值转换失败返回 `NUMERIC_FAILURE`。失败不产生正常指数计划，按 F09.7/I03 停止并重建。

normalization 属于一次性 source-frame 音频计划，后续 SNR feasibility trial 只读取固定结果，不重复执行或推进其上游状态。reset 和 EOF 沿用 F09.7 的完整帧时钟，不新增一轮 normalization flush。

旧研究记录的72个合成配置组合仍只是既有模型trace，不宣称全输入域独立normalization等价。19个必要非秘密标量现已由授权provider供应；D06-E的32项继续由独立数学规则供应。fresh实际参数复审状态见F09PA局部审查记录。
