# F04F 四路扩散滤波与实例生命周期

本条款定义 F04A.4 的四路 filter，不替代 prefix gain producer。生产参数的接口与取得条件全部列明；未确认的编译参数值暂留研究侧。合成参数只能用于局部验证。

## F04F.1 参数与冷状态

采样率固定 48000 Hz。构造接口需要 coefficient[4][4] F32，和十项 F32 配置：ring_seconds、bank_total_seconds、bank_geometric_base、bank_exponent_step、gain_seconds、slow_seconds、fast_seconds、input_seconds、envelope_epsilon、envelope_limit。分别登记 D11-DC 和 D11-DP；DC由F04FM、DP由F04FCM提供逐位等价数学生成路线。原研究表不复制，其他配置范围不得以合成值补全。

配置全部 finite；时间、base、epsilon、limit 为正。构造出的环和四个 bank 长度均须在 1～4096，超过时报 SIZE_LIMIT。数学构造的分母必须为 finite 正值，生成结果必须 finite，否则报 PARAMETER_DOMAIN。没有已确认参数时报 PARAMETER_UNAVAILABLE。本接口不会提供静默缺省值。参数取得条件见 PARAMETER-INTERFACES.json。

令 Fs=R32(48000)。环长度为 trunc((Fs⊗ring_seconds)⊕R32(0.5))。对于 stage s=0～3，r_s=R32(pow64(base,R32(s)⊗step))；按 s 递增分别累加 r_s 为 F32 分母 T。长度 L_s=ceil(((Fs⊗bank_total_seconds)⊗r_s)⊘T)。每个 bank 是 L_s×4 项 F32，索引 p_s 独立。

时间系数函数 C(t)=R32(exp64(R32(-1)⊘(Fs⊗t)))。gain 两条曲线都使用 C(gain_seconds)。slow/fast 保存三元组 (C(t),1⊖C(t),accumulator)。input smoother 保存 (C(input_seconds),accumulator)。exp64/pow64 指数学结果最近偶数舍入到 binary64，再转 F32；原系统数学库的全输入 ULP 等价未证明。

冷状态：四个 bank、延迟环、所有包络与 input accumulator 为正零；所有索引为 0；gain0/gain1 为 1；fade_pending=true。每个活动 tail 独立拥有整份状态，不共享 bank 或 ring。

## F04F.2 首块淡入与检测信号

单次 filter 输入是 N 项 finite F32，0<=N<=4096，输出四路各 N 项。复制输入为 x，禁止原地修改调用者 PCM。

fade_pending=true 时，h=floor(N/2)。若 h>0，对 i=0～h-1 执行 x_i=(R32(i)⊘R32(h))⊗x_i，剩余样本不变。这意味着首次块不能任意拆分；淡入长度取实际处理块的一半。处理结束时 fade_pending=false，包括 N=0。当前音频 lifecycle 的首次块固定为 3072 项，不使用空块启动。

检测信号 d_i=(x_i⊕a)⊗input_coefficient；随后 a=d_i⊖x_i。后续延迟环保存 x_i，而不是 d_i。包络控制用 d_i，音频 all-pass 用延迟后的 x_i。

## F04F.3 两条增益曲线

对每个样本 i，先更新 gain1=(gain1⊖1)⊗gain_smoothing1⊕1，再同样更新 gain0。括号内每个 F32 运算分别舍入，不能融合。

level=R32(abs(d_i))⊕epsilon。fast 先更新，slow 后更新：每项包络 E=level⊗complement⊕coefficient⊗old_accumulator，并把 accumulator 保存为 E。若所得 E 等于零，仅将本次用于比较/除法的值替换为 epsilon；保存的 accumulator 仍为原 E。

如果 fast⊗gain0 > slow⊗limit，则 gain0=(slow⊗limit)⊘fast。随后计算 gain1_limit=(fast⊗limit)⊘slow；如果 gain1>gain1_limit，则 gain1=gain1_limit。两个比较都是严格大于。当前样本保存更新后的 gain0/gain1，供下一节同索引使用。

这两条曲线不分别对应四路。四路共享同一条 gain0 输入曲线和 gain1 输出曲线，但每级每 lane 的 bank 值独立。

## F04F.4 延迟环与四级 all-pass

对样本 i，先读取 delayed=ring[read_index]，再写 ring[write_index]=x_i。分别将 read/write 索引加 1，到环长度时归零。即使两个索引相等，仍先读后写。

四路初始 v_l=delayed⊗gain0_i。按 stage 0～3 更新。令 q_l 为当前 stage 的 bank[p_s][l]，c_l 为 coefficient[s][l]：n_l=v_l⊕(q_l⊗c_l)，新 v_l=q_l⊖(n_l⊗c_l)。写回 bank[p_s][l]=n_l，然后 stage 索引前进一次并环绕。四个 lane 的旧 q 必须在本 stage 写回前读取。

最终输出 y_l=v_l⊗gain1_i。输出保存为 planar lane0～3 的 F32。使用 N00 的独立 F32 加、减、乘、除，不使用 FMA，不重排阶段或样本顺序。返回完整 next_state 和四路输出；输入 state 是不可变快照。

## F04F.5 Tail 生命周期与两半音频

活动条件由 F04A.4 定义：object audio diffuse 非零、width>0、quiet=0。每 callback 先移除所有不活动 tail 的实例和 previous_filter_outputs。再次活动时创建新冷实例，不能恢复已删除的 state。

冷实例按顺序处理 previous_PCM[1536]||current_PCM[1536] 一次。四路输出各拆成 previous/current 两半，最后保存 current 半。连续活动实例只处理 current_PCM[1536]；本次 previous 半取该实例上 callback 保存的 current 输出。暖实例缺少这四路缓存时报 STATE_INCOMPLETE。

previous_PCM 为上一 callback 的 raw 输入，不是窗乘积、regular 缩放结果或 previous prefix。冷启动 PCM 为正零。EOF 的 cached callback 按 F02 继续处理同一个已填充输入，不能统一换成零。实例创建、删除和重新创建不改变其他 tail 的状态。

## F04F.6 检查、失败与局部验收

计算前检查参数、bank/环长度、索引、finite 样本、四路形状和缓存完整性。所有 state 数值也须 finite。计算后检查完整 next_state 和四路输出 finite；不满足时拒绝提交并执行 I03 fail-stop。本条款的 preview 返回新状态；跨 callback 和其他模块的全部实际提交点仍由 Q14 跟踪，不能据此声称全链自动回滚。

`../test-vectors/DIFFUSION-LOCAL-VERIFICATION.json` 使用显式合成状态，验证零系数的可手算延迟、暖处理分块不变、首块淡入，以及输入不变性。该报告不提供实际构造配置；固定参数由F04FM/F04FCM的独立文件提供。真实ADM、prefix gain全部依赖及整链验收不因局部报告通过而关闭。

## F04F.7 验收状态摘要协议

`DIFFUSION_STATE_V1`仅是本包独立测试的语义序列化，不是原worker/native结构。按以下顺序连接，整数U32LE、浮点F32LE；不得写主机padding、指针或JSON文本。F32保存位模式与signed zero。

1. 8-byte ASCII `DSTATE1` 后一个NUL；sample_rate U32，fade_pending U32（0/1）。
2. coefficient[4][4]，stage-major/lane-minor，16项F32。
3. stage0～3各依次：bank_length U32、bank_index U32、bank[entry][lane]的全部F32，entry递增/lane0～3。
4. ring_length/read_index/write_index三个U32，随后按物理数组index递增的全部ring F32，不旋转成读取次序。
5. 12项F32：gain0、gain1、gain_smoothing0、gain_smoothing1、slow coefficient/complement/accumulator、fast coefficient/complement/accumulator、input coefficient/accumulator。

对整个字节串计算SHA-256。十项不可变生成配置由参数文件SHA单独绑定；上述摘要覆盖F04F的全部动态state及随state携带的系数/长度。冷状态和每个连续next_state的expected/actual摘要见两份math-generation验收报告；单个equal布尔值不替代摘要比较。
