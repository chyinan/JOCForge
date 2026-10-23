# Q06 selected LFE gate 与 mantissa 边界

## 1. 范围

本规格关闭固定 48 kHz、1,024 kbps、5.1、6 blocks profile 的两个 Q06 接口：LFE normalization error gate producer，以及 PRIMARY/LFE 进入 mantissa 路线前的系数边界。

本接口只接收实现自有的配置、F32 系数和指数行。不读取 PCM、歌曲缓存、私有 key、认证状态或 native 输出。

## 2. LFE gate producer

gate 是独立的 signed I16 配置字段，不从五路 FBW gate 或系数内容推导。

programme 初始化先把包含 LFE gate 的两个相邻 I16 一次清零。只有以下三个条件同时成立时，后续配置 writer 才能把 LFE gate 写成 1：

- AHT enable 为 true；
- programme 使用 6 blocks；
- writer 的判定值恰好为 1。

其他情况保持 0。selected profile 的 AHT enable 固定为 false，因此正常 producer 只输出 gate=0。这个值来自配置状态初始化和 writer 条件，不是 CLI 默认值，也不是从 `available_budget_words<896` 的 FBW gate 复制。

隔离实现的公开 profile 入口只接受 AHT=false 的 selected profile并直接产生 0。上面的条件 writer 是有界研究 oracle，用于证明 0 的来源；本规格不把 AHT=true 扩展成受支持产品配置。

normalizer primitive 接收所有 signed I16 gate。gate=0 使用有限误差门限；任何非零 gate 都选择 zero-factor 路径。非零 gate 只用于原语边界测试，selected producer 不可达。

## 3. 合法输入域

- PRIMARY 扫描原语：0～253 个 U32 F32 位模式；selected FBW 消费行固定为 217 个。
- LFE MAIN row：恰好 7 个 U32 F32 位模式。
- 每个 coefficient 必须为 finite binary32。bool、NaN、infinity、F64 未先舍入值和超出 U32 的整数拒绝。
- LFE normalization raw row：恰好 7 个整数，每项 0～24。
- span 固定为 1；option 只接受 exact integer 0 或 1；gate 只接受 exact signed I16。
- selected source producer 只接受固定 profile、AHT=false、6 blocks。其他 profile 返回 `LFE_GATE_PROFILE_UNSUPPORTED`。

## 4. PRIMARY 与 LFE clip

PRIMARY 和 LFE MAIN 都调用同一个原位系数扫描原语。LFE 调用 count=7，PRIMARY 使用自己的有效 count。

对每个 finite F32 位模式：

1. value<-1 时写 exact F32 -1，word=`bf800000`。
2. value>1 时写 exact F32 +1，word=`3f800000`。
3. 其他值原位模式不变。等于 ±1 不换词，signed zero 和 finite subnormal 保留。
4. 从 clipped word 计算 PRIMARY raw exponent。
5. 后续 mantissa consumer 必须读取 clipped staged coefficient row。

因此 LFE 超范围系数不会原样进入 mantissa quantizer。它与 FBW PRIMARY 使用相同的严格比较和端点。指数计算与 clip 是同一个 staged mutation 的两个输出。

## 5. 指数规则

令 `w` 为 clipped U32。`d=W32(2*w)`。`e=(d>>24)-126`，但 d=0 时 e=-127。`h=clamp(-e,0,60)`；h=60 时改为24；最后取 min(24,h)。

这个规则不使用 abs 或 log2。`+0/-0` 都得到24，但 clipped word 保留零符号。`3effffff` 得到1，`3f000000` 得到0。

## 6. 错误与 state commit

所有 shape、type、finite、profile 和 gate 域检查在写 staged coefficient bank 前完成。

- gate/profile 错误：`LFE_GATE_INPUT_ERROR` 或 `LFE_GATE_PROFILE_UNSUPPORTED`。
- coefficient shape/type/nonfinite：`MANTISSA_INPUT_ERROR`。
- raw exponent或 normalization 输入错误：`EXPONENT_INPUT_ERROR`。
- F10 leaf 的有限中间量、数值转换或 radix-five delta 超域：`NUMERIC_FAILURE`。F09LB adapter逐字透传该码，不使用另一个 `EXPONENT_NUMERIC_FAILURE` 别名。

成功时，clipped coefficient row 与指数结果一起提交。任一后续 source stage 失败时不提交 staged row，programme 进入 retired。失败后继续调用拒绝；完整 reset 恢复 cold gate=0、空 coefficient bank 和未 retired。

上述错误字符串是隔离接口的规范错误码，不声称等于 DLL 的消息或数值码。入口域错误不写 state；进入 source transaction 后的下游失败保持旧 coefficient bank、零提交并进入 retired。

## 7. 验收

向量必须覆盖 selected gate=0、三个不可达 writer 条件、gate=-32768/-1/0/1/32767、zero-row gate0/非零第二轮差异、±1 等号、±1 相邻越界、最大 finite、signed zero、subnormal、PRIMARY/LFE 一致性、nonfinite/shape/type拒绝、下游失败零提交和 reset retry。

本规格不声称 AHT-enabled profile 已生产支持，也不证明真实整曲出现 clip。真实 render 和全 4,249-source 仍为独立 gate。

局部验证由独立 clean core 对照冻结 Python oracle：gate producer 来源证据、PRIMARY clip/exponent oracle 和 LFE normalizer。验证不运行 DLL，不读取 PCM，也不使用 key。
