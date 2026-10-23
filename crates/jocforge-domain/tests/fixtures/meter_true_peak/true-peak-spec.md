# Meter true-peak：已授权参数、插值状态及 programme 消费

交付范围：内部研究和内部实现。D12-TP-A6/B5 共11个非秘密功能值已得到逐项内部交付授权；不含原表文件、厂商源码/DLL、认证材料或公开/第三方分发授权。参数分类为 `AUTHORIZED_COMPILED_FUNCTIONAL_CONSTANTS`，不是独立数学设计生成的系数。此局部规格不代表全包 IMPLEMENTATION_READY=YES。

## F06TP.1. 参数与调用接口

参数文件 `parameters/meter-truepeak-authorized-v1.json` 给出 a[6]、b[5] 的确切 binary32 位模式和相同数值的十进制说明。每个 words 字符串为8位小写 ASCII hex，解释为 U32 位模式；F32LE 序列化时按该整数的低字节先写。以位模式为身份依据，不能重新从显示精度不足的十进制文本生成近似值。

参数展开为 `A[j]=a[min(j,11−j)]`（整数 j=0..11）；`B[j]=b[min(j,9−j)]`（j=0..9），`B[10]=B[11]=正零`。这是已授权值的镜像和填充，不能称为独立滤波器设计。A/B 各12项，顺序是滑动窗口内 tap 索引递增；展开后的 A 后接 B 共96个 F32LE 字节，SHA-256 必须为 `f4878bb808821078480754d676a94191954771757c07010982e97be8c571e84c`。11个独立值的44字节身份为 `f9bdfb75951c9660c2d5498c1997e234880ac6b4f27bcb64f3e02af91d711cb4`。

选定调用 profile 固定为48 kHz、六路 F32 `[L,R,C,LFE,Ls,Rs]`；每次每路恰好1024个源样本。state 是六路各4628个 F32；cold/reset 为全部正零。LFE 的逻辑索引固定为3；不得把它替换为任意 ADM 物理轨号3。每路拥有独立持续历史，不从主编码器或参考文件恢复状态。programme state 另有各路 true/sample peak internal 最大值，共12项，从 `R32(−15/16)` 初始化。

调用输入、所有持有的 state 和参数均要求 finite F32；保留两种零的位模式。参数必需，缺失时按 I03 报 `PARAMETER_UNAVAILABLE`；错误形状、非有限或输入转换溢出按 I03 报 `TP_INPUT_ERROR`，不能返回可用 next state。完成计算后验证中间值和输出有限，再提交完整 next state；运算中间值非有限或溢出报 `TP_NUMERIC_FAILURE`；失败不提交部分 state，按 I03 停止并丢弃整个 pipeline，完整 reset 后才重新开始。本包输入域和提交规则不认证厂商在非法输入时同样失败或回滚。

以下 `⊗`、`⊕` 每次分别最近偶数舍入至 binary32，遵守 N00；没有 FMA。常量1/4精确，noise 为 `R32(10^-20)`。不使用 double 累加、不同 SIMD 横向顺序或别的系统 resampler。

## F06TP.2. 12-tap FIR 运算

对数据 D、输出长度 C，要求 len(D)≥C+11。对 i=0..C−1、lane j=0..3：

`T[j]=((D[i+j+4]⊗coef[j+4])⊕(D[i+j]⊗coef[j]))⊕(D[i+j+8]⊗coef[j+8])`。

本项输出 `(T[0]⊕T[2])⊕(T[1]⊕T[3])`。先相加第4～7与0～3 taps，再加8～11 taps；最后 lane0/2 与 lane1/3 配对。不能改为十二项顺序 sum 或 lane0/1 左结合。

## F06TP.3. 一路两级4×插值

state 索引从0开始，长度4628；第二阶段起点 S=1547。先复制旧 state 成为 working state，source 保持只读。所有 slice 的结束索引不含。

1. 对 i=0..1023，写 `state[11+i]=(source[i]⊗(1/4))⊕noise`。
2. 从 state 索引0运行 `FIR(state,1024,A)`，得 first[1024]。
3. 对 i=0..1023，依次写 `state[S+9+2i]=first[i]`、`state[S+10+2i]=state[6+i]`。后者从第一层尚未移位的当前 state 读取。
4. 复制 `state[1024:1035]` 的11项到 `state[0:11]`，使用完整复制语义，不能被重叠覆盖改变读取内容。
5. 从 `state[S:]` 运行 `FIR(...,2048,B)`，得 second[2048]。对 i=0..2047，`output[2i]=second[i]`、`output[2i+1]=state[S+5+i]`；output 长4096。先形成完整 output 再移历史。
6. 复制 `state[S+2048:S+2057]` 的9项到 `state[S:S+9]`。未被上述步骤写入的 state 元素保持原 working state 的值。提交整个4628项，不只是首11和第二层首9项。

六路按逻辑索引独立处理。LFE ordinal=3 跳过上述插值，测量数组为原1024输入，factor=1；其整个4628项 state 按位保持。其余路测量数组为4096插值结果，factor=4。FIR 的 quarter-scaled数组仅为测量中间数据，不写回 meter 空间输出或主编码 PCM。

## F06TP.4. 每块峰值、位置和 programme 最大值

对测量数组取绝对值最大量 max_abs，相等峰保留最小数组索引 k。`raw_peak=max_abs⊗R32(factor)`。输出该未经过指标 clamp 的 raw_peak；本块的 grouped index 为 `((k//4)*4)//factor−7`。factor=1 的 LFE 分支也先按4样本组取整。该值允许负数，是局部块分组位置，不是已经加上 programme 块起点的全局位置。

raw_peak 调用 F06L.5，得到 true-peak internal/host；不改写 raw_peak 或 FIR 历史。sample peak 同时从完全相同的原1024输入调用 F06L.4，得到最大绝对源样本、最早索引与 internal/host。两种指标的不同 clamp 规则保留，不能把 sample-peak 的 unity clamp 加到 true-peak FIR。

各路 programme 最大值每块更新 `tp[r]=max(tp[r],block_true_internal[r])`、`sp[r]=max(sp[r],block_sample_internal[r])`。EOF 的最终 host 分别为 `max(tp)⊗128`、`max(sp)⊗128`。所有比较和输入指标为 F32；不使用参考歌曲峰值预置 state。F06L 原有大幅度 log 分支的数值准入/有限性约束继续适用，本局部验收不把其未核验边界宣称全部通过。

## F06TP.5. 共同1024前缀、首尾和 reset

公布样本流和延迟/有效账本遵守 F06W.1。normal callback H=1536，末次 source 不足处补正零；独立 meter 空间延迟3104，三次 drain 的音频输入为零。空间 metadata 安装由前端规格负责，本条不凭音频账本猜测。

公布六路持续送入同一个 measurement ring；只在各路都有完整1024样本时消费。true/sample peak 与 K-filter 使用完全相同的 `C=floor(processed/1024)×1024` 前缀。EOF 剩余 processed−C 保留不消费，不补成一块、不单独为 FIR 追加零块；finalize 只取已有统计结果，不重新测量残余 PCM。插值状态跨完整块持续，但不为了让所有源尾到达迟延峰值位置而额外 flush。重开独立 programme 时完整清零 FIR state，同时重置12个 programme maxima。

例如 N=6522906 时按 F06W 公布6523392、消费6522880；剩余512个公布样本不进入 peak/K-filter。这个例子只验证调度公式，不预置真实输入尺寸或测量结果。

## F06TP.6. ID1 消费边界

独立冷 meter 测量完整前缀后，最终 true_peak_host 交给 ID1 configuration 的 field7，按已交付 half-dB quantization 编码。对应 timer4 的周期为24000源样本，缓存接受更大的有效 true-peak code，在该 timer 刷新时提交给 ID1，再走 metadata FIFO/EMDF；不按本块 FIR 输出逐 callback 直接重写 wire 值。使用相同整首测量结果完成 timeline 和 assembler 的一致性绑定。

选定 none DRC 链的 selected loudness 来自 energy/ITU/job；这个整首 true-peak scalar 不进入 dialnorm selector。DRC 自己的逐帧 `peak_inputs_six` 是另一音频统计通路，不能拿这里的最终 true_peak_host 替代，也不能因 none profile 省略该通路。FIR 输出未作用为主 PCM gain。此数据流边界不保证所有第三方解码器都以相同方式处理 ID1 metadata，播放器验收仍依既定目标执行。

## F06TP.7. 可独立重建的验收向量

`test-vectors/METER-TRUEPEAK-AUTHORIZED-VERIFICATION.json` 只使用上述已授权参数文件。每个 input 的 recipe 构造基底1024，再按 `[1,3/4,−1/2,1/4,−3/2,2]` 分别乘出六路，每次乘后 R32；不能把 expected state/output 当作输入。

序列0为8个连续块，从全正零 state 和12项−15/16 maxima开始，之后各实现沿自己实际 state推进。基底依次为全正零、首样本unity impulse、末样本−3/4 impulse、全零继承历史、模整数有理序列、交替±1/2、全负零、noise下方的小幅序列。第9个案例独立reset，仅LFE state按显式有理式构造非零，验证 bypass 完整保留该路，其他路仍cold。

recipe7 的1e−24先最近偶数舍入成binary64，与整数小值相乘也舍入至binary64，再R32；因此跨语言实现必须保留该 input 配方的精度顺序。其他基底/scale均为明确 dyadic 有理数。source_sha256 按rows/sample索引的F32LE验证输入身份。

每个 expected_output_sha256 对应该路整个测量数组：非LFE4096、LFE1024个F32LE。完整 next_state_f32le_hex 按六路/4628-word索引编码，恰好111072原始字节、222144小写ASCII hex字符，没有header；不得将该hex用于下一案例seed。sha256核验同一完整字节序列。peak projection 的 *_word 字段是十进制 U32 F32位模式；factor/index为普通整数。expected_programme_after含12个最大值词和2个最终host词。

9例共193536个输出词、249912个完整状态词、540个峰值字段、126个programme字段逐项比较全部一致，first_difference=null。一个独立scalar展开与冻结数学模型分别从input构造、持续自己的实际state；参数runtime供应仅为授权11值加镜像/zero-tail，不读取原参数表或原生输出。

这些是生成输入上的局部算法/生产参数身份验收，不是新真实ADM渲染、主编码流或全播放器测试，不宣称任意finite输入已穷尽核验。干净结果报告为 `test-vectors/METER-TRUEPEAK-AUTHORIZED-RESULT.json`；r1 无研究上下文独立审查已通过：9 输入身份、54 完整输出哈希、249912 状态词、540 峰值字段及126 programme字段全部匹配；错误路径和全有限输入域未穷尽验证。
