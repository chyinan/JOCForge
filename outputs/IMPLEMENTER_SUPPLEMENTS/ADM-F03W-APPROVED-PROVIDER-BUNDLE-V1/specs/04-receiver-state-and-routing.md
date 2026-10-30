# F04R Receiver状态和六路处理顺序

本文件定义Type10之后的16输入→6输出路线、cold state及非QMF阶段。QMF 的折叠、延迟、FIR、重排和合成见 [F04Q](04-qmf-analysis-and-synthesis.md)。FFT/descriptor 参数与 target coefficient producer 仍有待补项目，不用这些未完成部分替代已定义步骤。以下六路顺序统一为[L,R,C,LFE,Ls,Rs]。

## F04R.1 完整cold state

配置为48k、16个输入item、每item1536 F32、6输出lane、analysis width64。输入按F04A.9为direct LFE后接15 component，target updates是按input row索引的6项F32向量；每callback只覆盖给出的row.target，保留该row.current/slope/remaining；未给update的row保持旧target。目标向量必须由独立control producer生成，不由音频参考倒推。

| 状态 | 数量/维度及cold值 |
|---|---|
| preconditioning scalar | 16×F32正零 |
| row coefficient interpolation | 每16row有current/target/slope各6×F32正零、remaining6×U32零 |
| transform delay | 6×32样本正零 |
| first ring | 6个225样本正零，write=224/read=0 |
| LFE post-filter | 4节各2项F32正零，共8项 |
| analysis histories | 5row，每row9×64样本正零 |
| QMF delay histories | 2row，每row10×128 F32正零 |
| QMF band0 FIR histories | 2row各40 F32正零 |
| synthesis histories | 2row各576 F32正零 |
| temporal0 | span1217、3lane、每lane2×span样本正零，read=span/write=0 |
| temporal1 | span63、5lane，其他初始化同上 |
| temporal2 | span900、1lane，其他初始化同上 |
| temporal3 | span256、6lane，其他初始化同上 |
| reserved temporal4 | span896、15lane，同样cold；当前本receiver路线不消费、保持原状态，不计入主音频延迟 |
| descriptor | tail_peak=0、hold=0、smoothed=0、exponent=2、reference=R32(2/48)、gain=2^-4 |
| post-QMF six-lane delay | 6×32样本正零 |
| second ring | 6个225样本正零，write=224/read=0 |

所有样本数组初始化正零；remaining为整数计数，不是F32。旧诊断入口未绑定target_coefficients的sentinel不参与当前计算路线；实际消费row.target，不把历史NotImplemented诊断入口当作已验证整链的运行入口。

## F04R.2 每callback的阶段顺序

1. 16row分别执行scalar preconditioning；更新本row独立scalar。
2. 安装当次target updates；清6×1536 accumulator为正零。按48个32样本block、16row、6lane递增执行F04R.4。
3. 6lane各执行32-sample delay、PG01 N32双端混合及256缩放，再执行first ring。
4. 只对lane3按6个256样本chunk执行四节成对post-filter，state跨chunk/callback携带。
5. 对lane0/1/2/4/5依次做analysis QMF，每lane24个64样本slot。对surround的2路QMF执行延迟/FIR处理并synthesis回1536样本；LFE不进入QMF。
6. 按temporal0→1→2→3顺序交换，lane集合分别[0,1,2]、[0,1,2,4,5]、[3]、[0,1,2,3,4,5]。在temporal3交换前，使用当时的scratch数据选择descriptor index。
7. temporal3结束后，6lane共同执行descriptor gain ramp，随后32-sample六路delay、逐样本0.5缩放和second ring，产出最终六路。

不能提前算descriptor peak再把temporal0～2挪到它后面，也不能把四段temporal delays合成一个全通道buffer。所有状态作为一个成功callback的next state提交；失败遵守I03。

## F04R.3 Scalar preconditioning

必要非秘密参数alpha、beta、tiny_threshold为finite F32；当前scale固定0.25。每sample x：a=alpha⊗x，residual=a⊖old_scalar，clipped=clamp(residual,-1,1)，next_scalar=(residual⊗beta)⊕old_scalar，output=clipped⊗0.25。

scalar更新用未clamp的residual，不用clipped。每处理完256样本，若-tiny_threshold<scalar<tiny_threshold，置正零；两边相等不清。零也落在严格区间内，故此步骤可把negative zero变为positive zero，与F07非零abs-sum清零条件不同。

每callback1536样本恰好6段，scalar不在callback开头reset。原研究参数的独立供值及来源仍需参数登记；缺参数时失败，不代入零或identity后宣称生产路径完整。

## F04R.4 32样本row/lane累加与ramp

每row/lane有current c、target t、slope s、remaining r。必要F32参数为magnitude_threshold、delta_threshold、smoothing。block0的active=1，其余47block active=0；本block duration=48-block_index。

max(abs(t),abs(c))<magnitude_threshold时，output和四项state保持，不减remaining。否则delta=t⊖c。abs(delta)>=delta_threshold且active!=0时r=duration；duration>=1时s=delta⊗smoothing。相同delta条件但active=0时保留r/s。abs(delta)<delta_threshold时r=0、c=t。

r!=0时，按sample j0～31，window=(j+1)/32（可精确F32），coef=(window⊗s)⊕c，output[j]=(coef⊗input[j])⊕old_output[j]；block结束c=s⊕c，r以U32减1。r=0时使用t作为整block系数，output=(t⊗input)⊕old_output，结束c=t。

target不因ramp推进而改变。小到不能进入magnitude gate的状态保留旧r，这不是整row reset。每block先row0、再row1等，每row按lane0～5；各lane保存自己的output旧值，不能把16row贡献重新分组归约或自动FMA。

## F04R.5 Transform delay、blend和225 ring

每lane对block b输出旧32项delay，而把本block accumulator完整复制成next delay。令W为PG01 N32窗，a5=a6=2^-8：对旧delay样本x[j]，left=(x[j]⊗a5)⊗W[31-j]，right=(x[j]⊗a6)⊗W[j]，output=(left⊕right)⊗256。相应乘加各自F32，不先合并互补窗。

每个225 ring逐样本先把输入clamp到[-1,1]后写到write，再读取当前read位置作为输出，最后write/read各加1 mod225。写/读次序是算法；cold224/0导致224-sample实际延迟，不能称为225。input是finite F32，零符号按条件式clamp保留；ring复制原始位模式。

second ring同样的225/224/0规则；它之前先执行32样本delay，再0.5乘法。两ring状态各自独立。

## F04R.6 LFE四节post-filter

必要coef为4×5 F32，每节[a0,a1,b0,b1,b2]，state[h0,h1]。定义q=(b0⊗a0)⊕b1、z=(b0⊗a1)⊕b2；U=[1,a0,(a0⊗a0)⊕a1,a1⊗a0]，V=[0,1,a0,a1]，P=[b0,q,(q⊗a0)⊕z,q⊗a1]，Q=[0,b0,q,z]。

每对[x0,x1]逐j0～3求v[j]=(((x1⊗Q[j])⊕(x0⊗P[j]))⊕(h0⊗U[j]))⊕(h1⊗V[j])。前2项输出，后2项作为下一对state。每节先处理完整256样本，其output交下一节。最后四节的state保存到下一chunk。

当前该post-filter没有F07的block末tiny-history清零步骤，不自行加入它。不能用逐sample一般IIR替换上述pair运算而声称所选数值路线相同。生产20coef仍需明确供值或完整替代生成规则。

## F04R.7 Double-span temporal交换

单实例参数span S、lane数L；每lane状态buffer长2S，read/write索引允许0～2S，cold read=S/write=0。输入N=1536样本。c=min(S,N)，write_first=min(c,2S-write)，read_first=min(c,2S-read)。

先保存输入最后c样本，以write开始写入更新后的状态buffer，越过2S部分从0继续。再把输入原前N-c样本移到输出从S开始的位置（使用原值快照，按memmove语义）；输出首c样本从刚更新的状态buffer在read开始读，越界从0读。这一阶段先write后read，不能使用旧buffer代替更新后buffer。

next_write：c==write_first时为write+c，否则c-write_first；next_read同样以read/read_first计算。若next恰好2S可保持该值，下一call第一段长度0并转到buffer0；不强行在本call对索引做mod。数组copy不做浮点运算，各lane同索引进度但buffer独立。

## F04R.8 Descriptor阶段接口及末段delay

descriptor选择输入是temporal0～2之后、temporal3之前的scratch。peak仅消费0/1/2/4/5；head范围[0,1280)、tail[1280,1536)，并含old tail_peak。选择、hold/release、阈值和gain table的完整算法另见F04D；不是R19早期仅index128的诊断分支。

gain应用在temporal3之后；首256样本逐sample使用当时current gain，再按固定delta推进；后1280样本用target gain。随后6lane把old32-tail接到current[:1504]，next32-tail复制current[1504:]；0.5缩放后second ring。

输出是1536样本六路及F04R.1每项完整next state。reset恢复全部cold数组/索引/descriptor，而非只清最终六路。QMF和descriptor缺口关闭之前，不能以本文件存在而将receiver整体标完整。
