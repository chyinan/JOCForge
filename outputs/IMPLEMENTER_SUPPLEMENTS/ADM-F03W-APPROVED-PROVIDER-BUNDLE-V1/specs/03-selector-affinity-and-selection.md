# F03C Candidate selector

本条款定义显式feature/position/response输入到选中source的selector。它不提供feature FFT、geometry response或U04 target优化。当前capacity C=15、feature bands B=24，候选数N1..48；无候选由F03G提前返回，不运行本selector。

## F03C.1 输入、精度与初始化

candidate_ids[N]为唯一递增的global source整数0..47；F[B][N]为非负finite F32 feature，positions[N][3]为finite F32，width[N]为非负finite F32，attributes[N]为U32，R[C][N]为非负finite F32 basis response，previous_ids[C]为调用前target IDs。数组必须来自同一callback的producer。width作为F32数值使用，不能把其原始U32当整数宽度。

当前F03P实际路径的width来自post-audio feature-source的feature_width，不是原始音频width；按candidate身份取位模式再解释为F32，值为正零。独立selector原语仍保留非零width输入测试域，不能把这些测试数值当作本路径producer输出。

以下记F_jb=F[b][j]、r_jk=R[k][j]。各四则按N00独立F32舍入，sum从正零按指定索引递增；不得FMA或pairwise归约。定义pow32(x,p)=R32(x^R32(p))，cos32(x)=R32(cos(已舍入F32 x))，按数学实数解释并最近偶数舍入。这是明确的数学数值路线；不承诺各平台近似libm与它逐位相同，目标音频/空间验收前不自动宣称生产替代通过。负power底数或非finite中间结果按I03拒绝，不用NaN比较选择source。

每次selector调用mask[C]=0、coverage[C]=正零，selected为空。它们不跨callback携带；只有previous_ids为上一target状态。记录每候选的初始ordinal，用于压缩后保留身份。

## F03C.2 Affinity 与 complementary flag

scale=sqrt32(R32(11/C))⊗R32(0.2)。对有序候选对(i,j)，distance按X/Y/Z顺序累加 (positions_j-position_i)²，每减、乘、加独立F32。

attributes_i!=attributes_j时a_ij=R32(0.001)。否则d=scale⊕width_j，ratio=distance⊘pow32(d,2)，v=min(1,ratio)，angle=(v⊗0.5)⊗R32(π)，a_ij=max(cos32(angle),R32(0.001))。width取column j，因此原始affinity可以不对称。相同候选的a_jj=1，不能用previous ID差异代替attribute相等判断。

complement矩阵D[N][N]先全正零。按i递增、j=i+1..N-1递增，若存在slot使r_ik>0且r_jk<=0，并另存在slot使r_ik<=0且r_jk>0，则v=max(a_ij⊗R32(0.95),R32(0.001))，同时写a_ij=a_ji=v、D_ij=D_ji=1。使用当时a_ij，不分别折减两个方向；对角不调整。

## F03C.3 初始support与固定normalizer

每候选 S_j=Σ_b pow32(F_jb,0.2)，按band递增。每slot Q_k=Σ_j ((r_jk⊗r_jk)⊗S_j)，按candidate递增。每band T_b=Σ_j F_jb，W_b=pow32(T_b⊕1,0.2)⊗2。

Q/T/W只在调用开始计算一次，在后续feature衰减和candidate压缩后保持不变。S随选择推进并随候选压缩。不能把每轮剩余feature重新求总量当作新的T/W。

## F03C.4 每轮primary score与slot gate

对每候选i，按j递增累加 U_ib=Σ_j(F_jb⊗a_ij)，每个j内部band递增。candidate_ids[i]出现在previous_ids任一项时，各U_ib再乘6；比较的是global source ID，不是attribute或内部ordinal。

score_i=Σ_b [W_b⊖(pow32(max(T_b⊖U_ib,0)⊕1,0.2)⊗2)]，band递增。然后按slot0..C-1扫描第一项同时满足：mask=0、(coverage⊗2^-23)⊕coverage<R32(0.6)，且存在candidate使r_jk>0并score_j>0。

找到slot k时，所有score_j变为(r_jk⊗r_jk)⊗score_j，再设mask[k]=1。没有符合slot时不改score/mask。不能先选candidate再决定该slot，也不能选coverage最低slot替代递增首个符合项。

## F03C.5 Primary tie与secondary选择

求primary maximum，按当前candidate顺序，数值相等保留最早项。tie集合包含score>=maximum⊖R32(0.01)的candidate，顺序保持当前ordinal。tie仅一项则选它；多项则求这些candidate的最大S，并选第一项S不小于该max。primary不是只取严格相等最大值，secondary也不是再按global ID排序。

无合法tie或非finite导致未定义选择时拒绝SELECTOR_CHOICE_UNDEFINED。选中后追加global ID、初始ordinal及当前attribute，不能用同位置的压缩前数组取身份。

## F03C.6 Feature衰减与coverage

设选中当前ordinal为h。先按candidate、band递增，F_jb=(1⊖a_jh)⊗F_jb。

再按candidate j递增：D_jh!=0时整个candidate的coverage与S更新都跳过。否则δ=pow32(1⊖a_jh,0.2)，remain=1⊖δ；按slot递增，仅当选中candidate的r_hk>0时增加coverage：

```
numerator = ((r_jk⊗r_jk)⊗S_j)⊗remain
coverage_k = coverage_k ⊕ (numerator ⊘ (Q_k⊕2^-23))
```

本candidate的slot处理完后S_j=δ⊗S_j。这里使用更新前S_j，slot gate测试的是selected r_hk，不能改成只看candidate j的response。D跳过S更新，但不撤销前面的feature衰减。

## F03C.7 Survivor与终止

feature衰减和coverage更新后，对每candidate按band递增求sumF，按XYZ顺序求与selected的平方距离。仅保留sumF>0.5且（distance>R32(1/62)⊗R32(1/62)或attribute不同）的candidate。距离threshold的两次F32舍入不可合并成R32((1/62)^2)，两者相差一ULP。

survivor顺序保持当前顺序。同步压缩IDs、width、attribute、position、F、r、S，并按同一survivor list取affinity/complement的行和列。Q/T/W、mask/coverage和previous_ids不压缩，不重算。

无survivor时返回CANDIDATES_EXHAUSTED及已有selected；否则下一轮。最多C轮；填满capacity时返回CAPACITY_REACHED，即使还有未处理candidate。输出selected_count和按选择顺序的ID/初始ordinal/attribute，以及每轮选择、tie、gate、mask、coverage、survivor诊断；不是按ID排序的set。

## F03C.8 提交与未完成步骤

selector成功输出initial target候选，交U04/installation；它本身不提交target owner、metadata history或JOC clock。失败时禁止把部分selected当作成功结果继续，遵守I03。reset恢复调用局部scratch，programme的previous IDs由owner单独reset。

验收覆盖attribute/width非对称affinity、complement对称覆盖、固定Q/T/W、previous-ID boost、slot gate首项、0.01 tie窗口、secondary最早项、D跳过coverage、survivor严格0.5/距离边界和全部数组压缩。完整feature/response生成、数值路线的真实ADM空间验收及异常域对应仍未完成，不以本文件存在将F03整体标完整。
