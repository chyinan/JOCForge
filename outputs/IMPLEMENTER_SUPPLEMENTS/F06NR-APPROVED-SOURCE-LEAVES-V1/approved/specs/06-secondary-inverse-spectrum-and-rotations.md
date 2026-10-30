# F06NI：secondary inverse256 的拆分、旋转与输出

状态：`SELECTED_SECONDARY_FFT_LOCAL_REVIEW_PASS_REAL_SONG_OPEN`。完整保留本层算法，不以identity测试桩替代产品FFT。实际FFT64/128见F06NF，全部旋转/twiddle/scale/permutation由 `parameters/secondary-inverse-math-v1.json` 的明确数学规则和位模式供应；局部r1 fresh独立审查已通过；真实整曲验收仍开放。

## F06NI.1 域与必要接口

输入X为256个有限F32未量化谱值；short为从F06NR的strict short_consumed==1导出的boolean。调用方先保留0..216并清217..255正零。该清尾属于F06NR；本数学原语自身可接受完整256项用于独立测试。

每个descriptor的N为complex数量：long N=128，short N=64。分别需要pre_a[N]、pre_b[N]、post_sine[N]三个有限F32数组。共576个F32旋转参数由上述数学参数文件供应；合成wrapper向量仍只验证外层，不是实际参数证据。post_sine的0项是正常被消费的参数；j=0的cosine另按明确1常量产生，不访问post_sine[N]。

FFT_N接口消费2N个交错F32并返回恰好2N个有限F32。它是F06NF实际固定FFT64/128操作网络，不能换成舍入顺序不同的库FFT。其额外参数radix_scale[4]、twiddles[1536]以及permutation128[8]/permutation64[4]（U16）均由同一数学参数文件供应；选定FFT128只消费每行twiddle j0..31，FFT64只消费j0..15。F06NR的window[256]另由 `parameters/ordinary-transform-math-v1.json` 供应，其与原静态window的差异不在此冒充消失。

必要值缺失报PARAMETER_UNAVAILABLE；shape、类型或short非法报SECONDARY_INPUT_ERROR；中间非有限/转换溢出报NUMERIC_FAILURE。失败按I03停用完整pipeline，不产生正常输出。这是独立接口契约，不证明原厂错误返回一致。

## F06NI.2 Descriptor 的pre/post计算

每次乘、加、减独立R32，nearest-even，无FMA；Neg翻转F32符号位并保留零符号。设descriptor输入V[2N]。

对j=0..N−1，front=V[2j]，back=V[2N−1−2j]：

- work[2j]=(back⊗pre_a[j])⊕(front⊗pre_b[j])。
- work[2j+1]=(back⊗pre_b[j])⊖(front⊗pre_a[j])。

调用实际FFT_N(work)，返回Z[2N]。对j递增，a=Z[2j]、b=Z[2j+1]、s=post_sine[j]；c在j=0时为精确F32 1，否则为post_sine[N−j]：

- out[2j]=((a⊗c)⊕(b⊗s))⊗2。
- out[2N−1−2j]=((a⊗s)⊖(b⊗c))⊗2。

Descriptor输出2N项，此层不clamp。每个索引写一次。数组为本次独立scratch，不跨块持有；持续history归F06NR/F08所有。

## F06NI.3 Long inverse256

对j=0..127，V[2j]=X[255−2j]⊗(1/2)，V[2j+1]=Neg(X[254−2j]⊗(1/2))。Neg发生在已R32的乘积上，不能将负零自动换正零。

调用long descriptor得T[256]。对j递增写D[j]=clamp(T[2j]⊗2)、D[255−j]=clamp(T[2j+1]⊗2)。clamp始终先max(−1,v)，再min(1,v)，结果为F32。这里的乘2与descriptor内的乘2是两次不同步骤，不能漏掉一次。D交F06NR overlap；该overlap不重复clamp。

## F06NI.4 Short inverse256

新建A[128]、B[128]。对j=0..63：

- A[2j]=X[254−4j]⊗(1/2)，A[2j+1]=Neg(X[252−4j]⊗(1/2))。
- B[2j]=X[255−4j]⊗(1/2)，B[2j+1]=Neg(X[253−4j]⊗(1/2))。

分别调用同一short descriptor参数，但独立scratch，获得T_A[128]/T_B[128]。对j递增：D[j]=clamp(T_A[2j]⊗2)、D[255−j]=clamp(T_A[2j+1]⊗2)、D[64+j]=clamp(T_B[2j]⊗2)、D[191−j]=clamp(T_B[2j+1]⊗2)。clamp沿用.3。四个写入域不重叠并覆盖全部256项；不能把A/B简单拼接或交换中间两个quarter。

## F06NI.5 局部配方与未完成项

`test-vectors/SECONDARY-INVERSE-WRAPPER-VERIFICATION.json`明确使用独立合成旋转：pre_a=1/2、pre_b=1/4、post_sine[j]=R32((j%7)/8)，FFT则为明确的identity测试桩。它们只检验本层pre/post、拆分、clamp及写入布局，不是生产参数或真实FFT，不作为独立实现运行时供应。

十例涵盖long/short、正负零、边界谱脉冲、幅度大于1的clamp和dyadic序列，共2560输出词与冻结数学wrapper在相同明示测试条件下逐项一致。某些primitive谱输入刻意含217..255非零值，不能视为上游217-bin清尾已经通过。

F06NF及 `test-vectors/SECONDARY-FFT-INVERSE-VERIFICATION.json` 已交付实际FFT网络、全部数学供参与连续forward→inverse→overlap合成验收；原表文件/源码未交付。无研究上下文r1局部审查已通过；真实整曲隔离音频验收仍开放。wrapper identity桩向量只保留为分层诊断，不再代表当前实际FFT供应状态。本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
