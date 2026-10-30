# F06W Meter 的输出、滤波和能量窗口时钟

本文件定义独立 meter 通路的逻辑样本调度、K-filter 运算与能量窗口衔接。六路空间渲染和 DV32 的全部必要参数仍未闭合。只允许本 profile 的 48 kHz 输入；不得消费主编码器历史或已有参考测量结果。

## F06W.1 独立输出样本账本

输入有效样本数 N 为正整数，callback 长度 H=1536。normal_callbacks=ceil(N/H)，末次 source callback 的未用位置补正零。空间处理延迟为 2H=3072，shared DV 延迟 32；总 latency=3104，drain_callbacks=ceil(latency/H)=3。meter 的三次 drain 输入为零，区别于主编码通路的四次 cached padded 输入。空间 metadata 在 drain 时的安装仍属于完整 meter frontend 的待处理绑定，不从本账本猜测。

startup=latency、pending=0、processed=0。按 callback index 0～normal_callbacks+drain_callbacks−1 处理。normal callback 先 pending+=H；drain 不增加 pending。skip=min(startup,H)，startup-=skip；available=H−skip，valid=min(pending,available)。valid>0 时发布 raw 六路输出中每行 [skip,skip+valid)；发布位置为当前 processed。然后 processed+=valid、pending-=valid。

结束时要求 startup=0、pending=0、processed=normal_callbacks×H。公布 samples 含最后 source callback 的正零 padding。输出六路顺序为 [L,R,C,LFE,Ls,Rs]，每路 F32；输入轨号、空间位置和渲染必须从独立 ADM/config producer 计算。该账本不证明公布 PCM 与未延迟源轨逐样本相同。

## F06W.2 Complete-1024 消费边界

把公布 samples 连续送入六路 PCM ring。measurement 只在每行均具有完整 1024 样本时消费一块。消费长度 C=floor(processed/1024)×1024。EOF 剩余 processed−C 样本保留为未消费 PCM，不补零生成额外 1024 块。统计 finalize 排空已有统计信息，不让这些尾样本重新进入 K-filter 或 peak。

该规则同时供 sample peak、true-peak FIR 和 K-filter 输入。样本数来自本次新计算的 N；不能从歌曲名、参考 AU 数或捕获结果反推。完整 meter frontend 和 getter finalize 状态仍待补齐。

## F06W.3 K-filter 参数、冷状态和运算

当前48 kHz/modern配置为2个stages，每stage有9×4 finite F32系数。第8行是4项cold state，前8行是消费矩阵。另有lower[4]、upper[4]，各lane lower≤upper。接口按stage、行、lane逻辑顺序，不传地址或原二进制资产。64项矩阵采用F06KG与parameters/meter-k-matrix-math-v1.json的公开基础参数和独立有理生成规则；原研究表留研究侧，不安装零表或忽略滤波。

六路各自拥有2×4 state，从第8行完整复制。选定8个cold词为正零；lower四项−1，upper四项+1，noise=R32(1/10^20)。17基础参数由parameters/meter-k-state-bounds-v1.json生成。64矩阵和8个cold词由F06KG生成，构成完整2×9×4接口；不重复附加row8。reset恢复完整cold。分段长度T≤1024且为8的倍数，六路相同；K-filter的4样本原语仍遵守更严格的energy分段约束。缺少必要参数时PARAMETER_UNAVAILABLE。

每行先 y[i]=input[i]⊗(1/8)。按 stage 0、1 递增处理，previous 从该 stage 的旧 state 复制。按四样本组处理：a[j]=y[offset+j]⊕R32(10^-20)。对每个输出 lane l，acc=a[0]⊗matrix[0][l]，然后按 j=1～3 更新 acc=acc⊕(a[j]⊗matrix[j][l])，再按 j=0～3 更新 acc=acc⊕(previous[j]⊗matrix[j+4][l])。四个 acc 替换当前 y 组。

该组完成后 previous=[a[3],a[2],acc[3],acc[2]]。stage 的全部组完成后才提交 state[l]=min(upper[l],max(lower[l],previous[l]))。这个 clamp 不重写已生成输出，也不在每组之间执行。下一个 stage 消费上一个 stage 的 y，重复同一 noise 注入和运算顺序。最后 output[i]=y[i]⊗(1/32)。所有操作按 N00 分别舍入，不融合乘加。

## F06W.4 SIMD energy 求和

每个分段滤波输出最多 1024 样本，长度是 8 的倍数。first[4]、second[4] 从正零开始；按每 8 样本一组，把前四项平方分别累加到 first lanes，把后四项平方分别累加到 second lanes。每次平方和加法分别 R32。

total[l]=first[l]⊕second[l]。power_sum=(total[0]⊕total[2])⊕(total[1]⊕total[3])。禁止改成一个普通顺序 sum、横向四项左结合或双精度求和。

## F06W.5 960 样本窗口

窗口长度 W=RNE_integer((R32(48000)⊗R32(2^-31/50))⊗R32(2^31))=960。六路各自有 mantissa=正零、exponent=48 的累计 state，共享 samples_in_window=0；K-filter state 独立持续，不随每窗口清零。

每个 complete-1024 输入块按实际剩余长度切分。take=min(本块剩余样本,W−samples_in_window)。按六路索引 0～5 依次运行 F06W.3 和 F06W.4，再以 power_sum 调用 F06L.7，更新对应行的累计 state。游标与 samples_in_window 均增加 take。1024 和 960 都是 64 的倍数，所有实际分段均满足 8 样本约束。

samples_in_window=W 时，对六路按索引调用 F06L.8，n=W。随后以固定权重 [R32(1/2),R32(1/2),R32(1/2),正零,R32(141/200),R32(141/200)] 调用 F06L.9，产生 all-six mantissa/exponent，立即送入 F06I。权重末两项词为 3f347ae1；不从原研究表复制，也不换成未量化的其他 surround 比例。

每个完整窗口完成后，六路累计 mantissa 恢复正零、exponent 恢复 48、samples_in_window=0。原 K-filter state 保留。EOF 不足 W 的已滤波/累计样本不生成新的 integrated 输入记录；不要使用残余样本数调用 F06L.8 后宣称原路径完整等价。

## F06W.6 调度向量与剩余边界

METER-INTEGRATED-SCHEDULE-VERIFICATION.json 的 schedule_cases 只从 input_source_samples 构造账本。expected_spans/actual_spans 含 callback_index、phase、raw_frame_sample_begin、samples、valid_programme_sample_begin。phase 按 callback index 是否达到 normal_callbacks 决定。其他整数计数由 F06W.1、2、5 的公式产生。

本歌曲 N=6522906：公布 6523392 样本，消费完整 1024 前缀 6522880 样本；未消费 PCM 为 512（实际源尾 26、source padding 486）。完整 960 能量窗口为 6794 个，另有 640 个已消费但未提交能量窗口的样本。两种尾部残余须分别记录，不能合并成“全部测量”。

5个调度案例核验跨度和两层残余，不能替代真实渲染或完整meter验收。METER-K-PARAMETERIZED-VERIFICATION.json的3块合成矩阵仍只验证参数化网络，不能作为生产参数。选定64项矩阵的独立供表现由F06KG定义，全词身份和6块连续替换已复核。F06KG 与本文件 .3/.4 的供参和消费连接已通过 r2 隔离独立审查，见 METER-K-MATRIX-LOCAL-REVIEW-STATUS.json；完整 meter 仍待验收。

该合成报告从生成的 2×4 cold state 开始。按 block_index 0～2 连续处理，samples 决定源数组长度；sample_rule 按 i=0～samples−1 生成输入，last_sample_override 非 null 时覆盖最后样本。矩阵按 synthetic_matrix_rule 生成；lower/upper/noise 取 F06W.3 的固定规则。每块的 expected/actual_output_sha256 是全部输出 F32LE 的 SHA-256；next_state_words 按 stage/lane 顺序；power_sum_word 是 F06W.4 输出的 U32 位模式。运行 state 只从实际计算推进，不能从期望词恢复。
