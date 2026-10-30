# F03F Window→feature matrix

本条款定义48个source各3072样本到24×48 feature及其跨callback状态。输入已经过F04A共享三角窗；这里不再乘一次窗，也不从音频参考缓存读取feature。所选数学FFT路线需与真实ADM空间/音频验收绑定，不要求原SIMD网络逐位复现。

## F03F.1 配置和cold state

当前L=3072、Fs=48000，FFT长度为不小于L的最小2次幂F=4096，half=2048。每source有vector[24]和scalar F32，cold全正零，独立48份；不共享source状态。输出feature[band][source]为24×48 F32，band-major。

输入flag pair为carry/suppress两个整数0/1，来自同代source flag producer。suppress优先于carry。reset恢复全部48份状态；每callback不能重建cold状态代替连续更新。

## F03F.2 明确的4096点数学FFT

REAL_FFT4096_RADIX2_MATH_V1：前3072项为输入F32，其余1024项正零，复数imag全部正零。按12-bit reversal重排4096个复数。Lstage=2,4,…,4096递增，group base按Lstage步进，j0..Lstage/2-1递增；根(c,d)=R32(cos(-2πj/Lstage)),R32(sin(-2πj/Lstage))。

每对a/b求tr=(c⊗b.real)⊖(d⊗b.imag)、ti=(c⊗b.imag)⊕(d⊗b.real)，上方为a⊕(tr,ti)，下方为a⊖(tr,ti)。读取同一旧a/b后写两位置，无FMA，无额外normalize。精确象限根用(1,+0)、(+0,-1)、(-1,+0)、(+0,1)，固定数据见 [feature-fft4096-math-v1.json](../parameters/feature-fft4096-math-v1.json)，4095对root已用100/200位实数角独立核验。

消费自然频率顺序bin0..2047，交错real/imag共4096项F32；本数学路线不消费Nyquist bin2048，也不把它塞进DC的imag槽。分析QMF的1/64缩放不适用于这里。当前原FFT表路线和此数学路线的真实音频验收仍需完成，不从数学DFT相同推定全部浮点结果相同。

## F03F.3 数学生成的权重和分组

参数见F03FC及 [feature-processor-math-v1.json](../parameters/feature-processor-math-v1.json)：coefficients[2048]、groups[24]正整数、fast_decay和slow_retain两个F32。sum(groups)=2048。它们只依赖L/Fs，programme不可中途换参数；改变配置需新建实例和cold state。

每bin power=(real⊗real)⊕(imag⊗imag)，weighted=power⊗coefficient。按groups连续切分bin0开始的2048项，各组从正零按bin递增F32累加，得到grouped[24]。group_count不是固定每组相同，也不是QMF/JOC parameter-band表。

## F03F.4 双向band平滑

alpha=R32(1e-4)。forward从previous=正零开始按band递增，previous=(previous⊗alpha)⊕grouped[b]，保存forward[b]。

backward从previous=正零、band23..0递减：smoothed[b]=(previous⊗alpha)⊕forward[b]，然后previous=(previous⊗alpha)⊕grouped[b]。注意递推使用原grouped，不用刚写的smoothed；不能把两pass变成普通对称卷积。

## F03F.5 每source状态更新

1. suppress!=0：vector/scalar置正零，不读音频，不先执行carry。
2. 否则carry!=0：每项vector=old.vector⊗fast_decay，再按band递增重新求scalar。不能直接old.scalar乘decay代替。
3. 否则读取完整3072样本并执行F03F.2～4，得到observed。current_sum按band递增求和。定义 threshold(v)：scaled=v⊗100、slack=scaled⊗2^-23、结果slack⊕scaled。

current_sum>threshold(old.scalar)或old.scalar>threshold(current_sum)时jump：vector逐项取max(old.vector⊗fast_decay,observed)，相等取observed。否则accept=1⊖slow_retain，vector=(observed⊗accept)⊕(old.vector⊗slow_retain)。两个100倍比较都严格大于，等号和slack范围落入blend。

最后scalar按band递增求新vector之和，向输出matrix[:,source]复制vector，并保留next state。observed原和不直接当新scalar；所有source按0..47递增，state只写自己的row。

## F03F.6 所有权、失败与验收

需要FFT的branch必须有本source当次完整音频。reset/carry不解引用未提供的音频。source/vector/输出stride必须同域；FFT输出为2048个复数，不接受错长度。finite输入与中间值要求按N00；失败后programme按I03停止。

本callback feature交F03G gate及F03C selector；F03U读取selector之前的完整候选feature，不用衰减残留。验收需覆盖reset优先、carry scalar重算、FFT零padding/尺度/频率顺序、group覆盖、双向递推、100倍相等与jump、连续状态和reset。原FFT/新数学FFT route及完整source flag/audio producer的整链闭合仍未完成。
