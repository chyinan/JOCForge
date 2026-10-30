# F08 参数化变换数值规格

算法运算网络已定义。当前目标选定PG02完整 `math-generated-v1` 参数数据，按明确公式提供全部字段；不要求与旧D01～D04表身份bit一致。旋转/排列/active butterfly与旧研究身份一致，窗/LFE三角项存在已报告差异；这些差异是选定provider身份的一部分，不是用零表代替必要算法。当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`；新路线最终编码、render 和 listening 属于下游独立实现验证及产品验收，仍为 `NOT_RUN`。

## F08.0 参数接口

参数供应者提供以下字段，所有浮点项用原始 U32 binary32 位模式表示，文件字节序小端。未取得任一必要字段时返回 PARAMETER_UNAVAILABLE，停止 programme；不代入默认零表。

| 参数 | 类型和维度 | 用途 |
|---|---|---|
| window | 256×F32 | 折叠窗；D01 |
| long_cosine、long_sine | 各128×F32 | 长 pre/post rotation；D02 |
| short_cosine、short_sine | 各64×F32 | 两路短 pre/post rotation；D02 |
| spectrum_permutation | 128×U16 | 长输出索引；短路使用偶数项；D02 |
| butterfly | 64×F32 | 分阶段 FFT 旋转；第2项另被 LFE FFT8 消费；D03 |
| lfe_cosine、lfe_sine、lfe_recurrence | 各8×F32 | 低频递推及旋转；D04 |
| lfe_permutation | 8×U32 | 低频八点选择；D04 |

所有值有限。spectrum_permutation 为0～127双射，偶数项组成0～63双射；lfe_permutation 为0～7索引，是否需要更强排列条件由实际参数身份约束。普通非秘密功能常数的内部使用与实现侧交付已统一授权；生产供应者只须逐项核验精确值、来源分类、消费范围、版本、维度和内容身份。标准几何或数学近似不能自动替代这些位模式。

## F08.1 状态及输入

每路输入256个F32样本。每个全带宽通道有独立 primary 与 companion 的128项历史；LFE也有各自128项历史。冷初始化为正零。每个块先读取完整旧历史，计算 next history；一个路的历史不能给另一通道或另一 bank 作种子。跨长/短切换保留本 bank 的历史。

本节 ⊕/⊖/⊗ 为 N00 的分别舍入 F32 运算，neg 为有限F32符号位翻转，包括正负零。任何式子均不得重新结合或自动融合。数组下标从0开始。

## F08.2 折叠窗算子

对 j=0～127，配对位置 t=255-j。令 a=x[j]⊗window[t]、b=x[t]⊗window[j]、c=x[t]⊗window[t]、d=window[j]⊗x[j]。

位置函数如下：j为偶数2i时，r=128+2i、q=129+2i、h=64+i；j为奇数2i+1时，r=127-2i、q=126-2i、h=63-i。这些 r/q 完整覆盖256项，h完整覆盖128项。

primary：fold[r]=a⊕b、fold[q]=old_history[h]、next_history[h]=c⊖d。

companion：fold[r]=old_history[h]、fold[q]=a⊖b、next_history[h]=c⊕d。

所有旧值来自块开始的同一历史快照。此算子不因 short flag 不同而改变状态维度。

## F08.3 Pre rotation

长路对 k=0～127，以 u=fold[2k]、v=fold[2k+1]、C=long_cosine[k]、S=long_sine[k]，产生 complex[k]=[neg(C⊗u)⊕(S⊗v),(C⊗v)⊕(S⊗u)]。

短路分别产生两个64-complex数组。对 k=0～63，第一路输入为 [fold[2k],fold[129+2k]]；第二路为 [fold[128+2k],fold[1+2k]]；均使用上式，但 C/S 改为 short_cosine[k]/short_sine[k]。

## F08.4 分阶段归一化 FFT

输入 M=128（长）或64（每条短路）的 complex 数组。stage s从0到log2(M)-1，H=M/2^(s+1)，G=2^s。每阶段按 g=0～G-1、p=0～H-1遍历；本 pair 的 complex 下标 A=2Hg+p、B=A+H。先取旧 [ar,ai] 和 [br,bi]，再写两个结果。

g=0 时 tr=(ar⊕br)⊗0.5、ti=(ai⊕bi)⊗0.5。

g=1 时 tr=(ar⊕bi)⊗0.5、ti=(ai⊖br)⊗0.5。

g>=2 时，C=butterfly[g-1]；g为奇数时S=butterfly[g-2]，偶数时S=neg(butterfly[g])。tr=((ar⊖(C⊗br))⊖(S⊗bi))⊗0.5；ti=((ai⊖(C⊗bi))⊖neg(S⊗br))⊗0.5。

每个 pair 写 A=[tr,ti]、B=[ar⊖tr,ai⊖ti]。pair 的旧 A/B 必须在两项写入前读完。stage 之间依次使用上阶段结果。这里的索引及专门的 g=0/1 分支属于算法，不能用通用 FFT 的数学等价式替换。

## F08.5 长 Post rotation

对 k=0～127，取 FFT[spectrum_permutation[k]]=[r,i]、C=long_cosine[k]、S=long_sine[k]。

primary 输出[2k]=(C⊗i)⊖(S⊗r)；companion 输出[2k]=(S⊗r)⊖(C⊗i)。两路的输出[255-2k]均为(C⊗r)⊕(S⊗i)。结果为256项，前217项进入普通编码。

## F08.6 短 Post rotation

两个64-complex FFT结果分别记为 A/B。对 k=0～63，取索引 spectrum_permutation[2k] 的 [ar,ai]、[br,bi]，以及 C=short_cosine[k]、S=short_sine[k]。

primary 输出[4k]=(C⊗ai)⊖(S⊗ar)，[4k+1]=(C⊗bi)⊖(S⊗br)。companion 这两项分别为(S⊗ar)⊖(C⊗ai)、(S⊗br)⊖(C⊗bi)。

两种 bank 的输出[254-4k]=(C⊗ar)⊕(S⊗ai)，[255-4k]=(C⊗br)⊕(S⊗bi)。两短路的结果按此方式交错，不拼成前后两个128项实谱。

## F08.7 LFE FFT8 算子

输入为16个F32，分为四组四项向量 V0～V3。所有向量加减均逐项执行 ⊕/⊖；下面每步右值先完整保存后再写左值。

1. 对 pair(0,2)、(1,3)，写 Va=旧Vb⊕旧Va、Vb=旧Va⊖旧Vb。
2. V3=[neg(V3[1]),V3[0],neg(V3[3]),V3[2]]。
3. 对 pair(0,1)执行步骤1的 pair 算子。
4. 把 V1 的后两项变为 [neg(旧V1[3]),旧V1[2]]，前两项保持。
5. V2=旧V2⊖旧V3，V3=旧V3⊕旧V2。
6. R=butterfly[2]。令 a=R⊗V2[2]、b=R⊗V2[3]，后两项改为[a⊕b,b⊖a]；另对V3以其旧后两项求a/b，改为[b⊖a,neg(a)⊖b]。
7. 每组旧[a,b,c,d]写成[(a⊕c)⊗0.125,(b⊕d)⊗0.125,(a⊖c)⊗0.125,(b⊖d)⊗0.125]。
8. V1从旧[a,b,c,d]变为[c,d,a,b]。

按V0,V1,V2,V3顺序展平，得到16项。0.125和0.5是算法的精确二进制归一化因子，不是未交付表的替代值。

## F08.8 LFE 低频递推

LFE采用 F08.2（相应bank）和 F08.3长 pre rotation。对g=0～15，收集k=0～7的pre complex下标 g+16k，按实/虚交错得到16项，执行FFT8，记Z[g]。

每个频率k=0～7独立处理：index=lfe_permutation[k]，prior=[+0,+0]，current=Z[15]的该complex。按g=14递减到0，逐part计算 next=((lfe_recurrence[k]⊗current[part])⊖prior[part])⊕Z[g][2index+part]；然后prior=旧current、current=next。

实部低频[k]=((current[0]⊖(prior[0]⊗lfe_cosine[k]))⊕(prior[1]⊗lfe_sine[k]))⊗0.0625。

虚部低频[k]=((current[1]⊖(prior[0]⊗lfe_sine[k]))⊖(prior[1]⊗lfe_cosine[k]))⊗0.0625。

## F08.9 LFE 七项输出及消费

对k=0～3，primary 输出[2k]=(long_cosine[k]⊗低频虚部[k])⊖(long_sine[k]⊗低频实部[k])；companion 为相反的乘积相减顺序。

输出位置/低频k/长旋转索引分别为(5,5,125)、(3,6,126)、(1,7,127)，这些奇数位置均用 (long_cosine[旋转]⊗低频实部[k])⊕(long_sine[旋转]⊗低频虚部[k])。

LFE不使用 fullband short branch。每个bank产出7项及独立next_history。POWER使用两bank，PRIMARY指数使用主bank。当前接受链的LFE mantissa路径保留变换主系数；原生超范围边界仍未证明，Q06不得关闭。

## F08.10 验收与失败

先以独立生成的 synthetic 参数检查操作网络、长→短→长连续状态、LFE两bank、所有位模式与终态；synthetic参数不得称为生产参数。生产参数可取得后，再实际核验生产长短/LFE向量和整首。

尺寸、类型、参数内容身份或有限性校验失败不得进入变换。计算失败采用 I03 fail-stop，不用未知/零资产继续。前端与编解码器全部reset范围仍需相关条款补齐，不把本算子的128项reset当作整个programme reset。

选定接口的规范错误码固定如下：输入块或history尺寸错误、short flag不是严格I1整数0/1、输入/state含非有限值时报 `F08_INPUT_ERROR`；必要参数字段缺失报 `PARAMETER_UNAVAILABLE`；参数文件hash/schema/profile、字段类型/长度、finite身份或permutation身份错误报 `PARAMETER_IDENTITY_ERROR`；运算中产生非有限binary32或转换失败报 `F08_NUMERIC_FAILURE`。这些错误均返回 `output_present=false`，不返回正常next history，调用方旧state逐字不变，并按I03停止/重建。

## F08.11 Nonzero composition 局部验收

局部规格见 `specs/08-nonzero-transform-composition.md`，input/result见 `test-vectors/F08-NONZERO-COMPOSITION-INPUT-VECTORS.json` 与 `test-vectors/F08-NONZERO-COMPOSITION-RESULT.json`。验收以非零合成输入连续执行3个FBW块的long→short→long和3个LFE块，固定window/fold、transform、前217/7项POWER与PRIMARY raw指数输入、独立bank state、完整reset和7个machine-readable拒绝结果。独立实现完成后与已验证纯模型post-compare 4,650个U32输出/history词，差异0。

该结果使 `math-generated-v1` 的局部非零composition为 `YES`。它不要求旧D01～D04 table identity，也不把生成provider说成旧表bit一致；最终整帧五FBW+LFE提交、编码/render、播放器与听感仍是独立验收门槛，whole implementation保持 `NO`。剩余边界见 `test-vectors/F08-NONZERO-COMPOSITION-GAP-AUDIT.md`。

r1 package-only独立复审登记见 `test-vectors/F08-NONZERO-TRANSFORM-LOCAL-REVIEW-STATUS.json`：17/17冻结身份、3个FBW long→short→long、3个LFE、primary/companion连续state、80个公开trace hash及preview、7项reset断言和7项machine error均独立重算一致，first difference为null。冻结报告中的4,650词只可确认其公开hash/preview身份与独立计算一致；冻结未交付4,650个expected词或已验证Python model，因此该数字不能登记为第二次独立逐词oracle replay。局部 `math-generated-v1` composition为YES；历史D01～D04 identity、整帧commit、render/player/listening与whole implementation仍为NO。
