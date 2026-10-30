# Secondary inverse 的 FFT64/128 数学网络

功能标识 `F06NF`。本规格提供 F06NI 所需实际 FFT，替换局部验收中的 identity 桩。网络无运行时历史、不执行归一化、不改变逆变换外层已有的两次乘 2。选定链路仅使用 N=64 或 128。所有加、减、乘都分别按 N00 转成 F32，禁止 FMA、结合或替换成不同舍入顺序的库 FFT。

## 参数与接口

输入是 2N 个有限 F32，交错排列 Re0,Im0,Re1,Im1,…。输出同为 2N 个交错 F32；输出索引全部写入一次。参数来自 `parameters/secondary-inverse-math-v1.json`：六个旋转数组、六行 twiddle、四路 scale、两个 permutation；共享窗口来自F08的 `parameters/ordinary-transform-math-v1.json`。错误输入报 `FFT_INPUT_ERROR`，缺供应报 `PARAMETER_UNAVAILABLE`，长度、字段或数值错误报 `FFT_PARAMETER_ERROR`，计算产生非有限结果报 `NUMERIC_FAILURE`；按 I03 终止本次流水线、丢弃未提交状态，完整 reset 后才可继续。参数必须先校验固定字段和长度，输入预检完成后才开始算；F32 输入转换后也必须有限。N 不在选定域则 `UNSUPPORTED_PROFILE`。输出无副作用。

六个旋转数组独立定义为 N128/64 各一套，j=0..N-1：

`A[j]=R32(sin((j+1/4)*pi/(2N))/2)`；`B[j]=R32(cos((j+1/4)*pi/(2N))/2)`；`P[j]=R32(sin(j*pi/(2N)))`。不能把 A/B 混同 F08 的前向旋转。

twiddle 的原数学域是六行×256：m=1,2,3，j=0..255，令 `q=(m*reverse8(j)) mod1024`、`theta=2*pi*q/1024`。行 `2(m-1)` 为 `R32(cos(theta))`，下一行为 `R32(-sin(theta))`。q=0,256,512,768 分别使用 `(1,+0),(+0,-1),(-1,+0),(+0,1)`；此处 +0 是精确符号位规定，不能靠一般 libm 近似替代。scale 四个 lane 都为 `R32(sqrt(1/2))`。permutation 为 `2*reverse_bits(g, log2(N/16))`。

生成精度必须保证最后 R32 的唯一舍入结果；配套生成词及哈希是完整供应/验收依据，不使用研究侧原表文件。六个旋转共 576 词、twiddle1536 词、scale4 词、permutation12 个 U16 已与研究后比较全域一致。实际 FFT128 只消费 twiddle 每行 j0..31；FFT64 只消费 j0..15。完整数学生成可供应全域，不要求实现复制未消费的存储尾部。

## 四路基础操作

以下每个大写变量是四路 vector；运算逐 lane，`+,-,*` 每次均 R32。所有输入在首次 store 前保留为独立值。`F4(r0,r1,r2,r3;i0,i1,i2,i3)` 定义：

```
p=r0+r2; q=r1+r3; u=i0+i2; v=i1+i3
d=r0-r2; e=r1-r3; h=i0-i2; k=i1-i3
result[0]=(q+p, v+u)
result[1]=(p-q, u-v)
result[2]=(k+d, h-e)
result[3]=(d-k, h+e)
```

这是 bit-reversed 四路输出次序 0,2,1,3。`B4` 为以下通用有 twiddle 四路操作；输入 r/i 四对以及 c1,s1,c2,s2,c3,s3（每个可为 scalar 广播或四路 vector）：

```
a=c2*r2-s2*i2; b=c2*i2+s2*r2
c=c1*r1-s1*i1; d=c1*i1+s1*r1
e=c3*r3-s3*i3; h=c3*i3+s3*r3
P=a+r0; Q=b+i0; D=r0-a; E=i0-b
U=e+c; V=h+d; W=c-e; Z=d-h
result[0]=(U+P,V+Q)
result[1]=(P-U,Q-V)
result[2]=(Z+D,E-W)
result[3]=(D-Z,W+E)
```

最终 stage 的 h 必须使用 `s3*r3+c3*i3`（交换两个 product 的加法顺序），其余同 B4。当前选定有限输入/功能参数域内已单独验证两者；仍按明确顺序实现。

## 初始 stage

定义 G=N/4（N64）或 N/8（N128）。k=0,4,…,G-4；从交错输入读取四个相邻 complex，令 `r_g[l]=input[2(gG+k+l)]`、`i_g[l]=input[2(gG+k+l)+1]`，l0..3。

N64：直接对 g0..3 使用 F4，结果按下述 store 保存。

N128：E=F4(r0,r2,r4,r6;i0,i2,i4,i6)，O=F4(r1,r3,r5,r7;i1,i3,i5,i7)。用 odd 四路差值：

```
R=r1-r5; S=r3-r7; I=i1-i5; J=i3-i7
a=(I+S)*scale; b=(I-S)*scale
d=(J+R)*scale; e=(R-J)*scale
u=b+d; v=b-d; w=e-a; z=a+e
block0=(O0.real+E0.real, O0.imag+E0.imag)
block1=(E0.real-O0.real, E0.imag-O0.imag)
block2=(O1.imag+E1.real, E1.imag-O1.real)
block3=(E1.real-O1.imag, O1.real+E1.imag)
block4=(u+E2.real, v+E2.imag)
block5=(E2.real-u, E2.imag-v)
block6=(E3.real-w, E3.imag-z)
block7=(w+E3.real, z+E3.imag)
```

store 每个 block g 的 real4 到 word偏移 `2G*g+2k`，imag4 到其后4词。N64 的 block 为 F4 的四对。初始 stage 从完整 input 读取、写全新 scratch，不原地覆盖。

## 中间 stage

先取 scratch words0..31，拆为四对 `(r_g=scratch[8g..8g+3], i_g=scratch[8g+4..8g+7])`，用 F4，再按每对 real4/imag4 连续写回0..31。注意：外层可能传入64词，但该 stage实际只读写前32词；32..63不在这里改写。

然后 g=1..N/16-1，以 words32g..32g+31 同样拆四对。六个 twiddle scalar 为六行在 j=g 的值，广播到四 lane。用 B4，将四对按 real4/imag4 连续写回当前32词。不存在跨 g 状态。

## 最终 stage 和输出排列

g=0..N/16-1，取当前32词 x。转置成 `r_t[l]=x[8l+t]`、`i_t[l]=x[8l+4+t]`，t,l0..3。每个 twiddle vector从其行读取 j=4g..4g+3。用最终版 B4 计算四对 bit-reversed结果 K0..K3；自然 band 顺序取 `[K0,K2,K1,K3]`。

对于 band b0..3、output_lane l0..3，vector_lane取 `[0,2,1,3][l]`；dest=`permutation[g]+b*(N/2)+l*(N/8)`。写 real 到 dest，imag 到 dest+1。无需另外整体 reversal、scale 或 conjugation。

## 验收及限制

14 个真实 FFT 案例共 2,688 输出词、10 个完整 inverse256 案例共 2,560 输出词、24 个连续 forward→inverse→overlap block 的 6,144 输出词与 6,144 前向/逆向状态词全部和冻结 Python 算法逐词相同。串接用 F08 数学窗口同时供应两条计算路线，包含217有效bin选定范围和256bin诊断范围，long/short 切换、冷初始化、连续 carry、零和负零输入。

另外对 FFT64/128 每个输入词分别放置 unity，384 个全域基向量案例共 81,920 输出词均逐词相同；其配方和输出 SHA 在同一向量文件中。生成参数还在80及160位小数精度分别复算，全2116 F32 词均一致；该检查是精度稳定性证据，不宣称形式化区间算术证明。

这不是原静态窗口逐词相等结论，也不是新真实整曲播放验收；此处没有 DLL/SDK 执行、私有 key、native PCM 或 native history 输入。r1独立局部审查760项检查零失败；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，真实整曲执行及产品播放验收仍为下游 `NOT_RUN`。
