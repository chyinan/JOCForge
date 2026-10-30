# F03N Rectangular NNLS 初始拟合

本条款定义15个basis observation、n个active target的初始非负拟合，0<=n<=15。目标是最小化 ||A x-b||²且x>=0；实际运行以以下活动集、舍入、rank与删除规则为准，不能只调用任意NNLS库并假定结果等同。

## F03N.1 输入、局部状态与精度

A为15×n F32 row-major，b为15项F32，全部finite。按每个F32的精确数值转F64，内部按column存储。每次调用独立初始化：x[n]=正零、dual[n]=正零、permutation=[0..n-1]、passive_count p=0、secondary_iterations=0。无跨source或callback的NNLS warm seed。

内部四则和sqrt为IEEE binary64最近偶数，每个乘法、加法、除法独立舍入，禁止FMA。序列归约按索引递增，括号决定顺序。rank factor=F64(R32(0.01))，正dual停止阈值=R64(10^-12)。最终x逐项R32交给后续F32能量路线。

n=0由outer入口直接跳过，返回0×48矩阵，不运行本算法。以下适用于n>=1。非finite中间值、零三角对角或可行性0/0是失败，不以任意epsilon替代。

## F03N.2 Householder反射的构造和应用

当前pivot p对应列c。若p+1>=15，不改变列并返回u=0，应用同样不动作。否则a=c[p]，s=max_{i=p..14}|c[i]|；s<=0时u=0且列不变。s>0时先计算inv=R64(1/s)，依索引从p开始计算 norm²=Σ(R64(c[i]×inv))²，每次乘/加分别F64。不能改成逐项直接除s。d=s×sqrt(norm²)，a>0时将d取负，否则保持正。保存u=a-d，仅覆盖c[p]=d，不改变tail。

应用至任意15项向量v：只有p+1<15、abs(c[p])>0和denom=c[p]×u<0时继续。t=u×v[p]+Σ_{i=p+1..14}v[i]×c[i]，逐项F64；t=0时不写向量。其他情况step=t×(1/denom)，v[p]=u×step+old v[p]，tail各v[i]=step×c[i]+old v[i]，乘加不融合。

构造拒绝candidate时只恢复旧c[p]；构造本来没有修改tail，不能重新计算或把整列恢复成调用最初输入。

## F03N.3 选择进入变量

当p<n且p<15时，对当前inactive permutation[p..n)逐个计算 dual[column]=Σ_{r=p..14}column[r]×rhs[r]。归约从p开始，按r递增，四项分组仍使用左关联总和，不用pairwise reduction。

从inactive permutation顺序扫描，选严格大于当前best的dual，best从0开始；相等保留最先访问位置。无正candidate或best<=停止阈值时结束当前活动集求解。

对candidate构造F03N.2反射。unorm=sqrt(Σ_{r=0..p-1}column[r]²)，空和为0。rank_increment=(unorm+abs(column[p])×rank_factor)-unorm，独立F64操作。只有rank_increment>0才将反射应用到rhs副本，并测试 trial_rhs[p]/column[p]>0。

rank或trial positivity未通过时恢复旧对角，candidate dual设0，再在当前inactive集合中继续找best，不立即终止所有candidate。接受时保留变换后的rhs，将candidate与permutation[p]交换，p增加1；把其反射应用到剩余inactive列，再将接受列从新p到14的tail置正零。candidate dual置0。

## F03N.4 三角求解

用当前接受列、permutation前p项与变换后的rhs求候选z。先复制rhs；按pivot=p-1..0递减，以column[permutation[pivot]][pivot]作对角，除出z[pivot]。然后按row0..pivot-1递增，z[row]=z[row]-z[pivot]×该列[row]，乘减独立F64。零对角拒绝。

z的前p项按passive permutation关联x，其余rhs项不当作额外变量。不能给inactive变量写入未求解的rhs tail。

## F03N.5 非负可行性与移出变量

每次检查z前，secondary_iterations加1；超过3n时返回ITERATION_LIMIT及当前已接受x，outer是否消费该状态另见F03S，不谎报收敛。

若全部passive z>0，将x[permutation[j]]=z[j]，结束本次secondary loop。否则对所有z[j]<=0，计算 alpha=-x[var]/(z[j]-x[var])；分母0拒绝。取最小alpha，相等保留最先passive位置，沿该边界更新全部passive x=(z-x)×alpha+x。最初移出的是选中位置的变量，其x置正零。

从该位置后面的passive变量依次左移，使用F03N.6消去移入列的两行，并对所有其他列和rhs做同一rotation。p减1，把移出的变量放到permutation[p]成为inactive。

明确索引：保存删除位置d、删除前passive数量p_old和removed=permutation[d]。依k=d+1..p_old-1递增，moved取当时permutation[k]，先写permutation[k-1]=moved；对moved列的row(k-1,k)构造rotation，写该列upper=sigma/lower=正零，再对所有其他列（原变量编号0..n-1，排除moved）和rhs的同一row对应用rotation。此轮完毕才设p=p_old-1、permutation[p]=removed。

随后扫描所有剩余passive x：若存在任何x<=0，当前所选路线移出最后一个passive变量，再做同样移位/rotation；重复直到剩余x全正或集合空。这里不是“找到第一个非正变量就移出它”。结束删除后重新三角求z，回到secondary检查。此规则须由边界向量验证，不能用一般库的清理策略代替。

## F03N.6 Givens rotation

待消去的一对为(a,b)。abs(a)>abs(b)时r=b/a，h=sqrt(r²+1)，c=abs(1/h)，a<0时c取负，s=c×r，sigma=abs(a)×h。否则b!=0时r=a/b，h同上，s=abs(1/h)，b<0时s取负，c=s×r，sigma=abs(b)×h。两者都0时(c,s,sigma)=(0,1,0)。

移入列的upper置sigma、lower置正零。其他列旧两项(U,L)变为(L×s+U×c, L×c-U×s)。rhs两项变为(U×c+s×L, c×L-U×s)。列和rhs的加法实参顺序如式所列，不融合。

## F03N.7 输出、状态与未完成范围

输出完整n项F64 x和n项F32 seed、permutation及p、dual、残差norm=sqrt(Σ_{r=p..14}rhs[r]²)、accepted/rejected/removal诊断和secondary iteration数。p>=15时dual全部置0。常规结束为SOLVED，超过上限为ITERATION_LIMIT；它们不是同一收敛结论。

rejected按每次实际拒绝的先后追加，重复拒绝同变量也重复记录，每项为{variable:原变量ordinal整数,reason:RELATIVE_RANK或NONPOSITIVE_TRIAL,passive_count:拒绝时p整数,rank_increment:F64数值}。rank_increment<=0为RELATIVE_RANK；rank通过但trial系数不严格正为NONPOSITIVE_TRIAL。记录发生在恢复旧diagonal、dual置0的同一candidate拒绝阶段，不按最终变量顺序重排。removals按实际移出先后追加{variable:原变量ordinal,passive_position:移出前位置}，包含后续last-pop。

本算法局部数据在每source调用后丢弃，不推进metadata、audio历史或JOC sequence。reset无额外NNLS缓存。rank rejection、相等candidate、非正trial、可行性移出/连续last-pop、iteration limit及rectangular维度均需验收。F03S负责能量分支、correction和后处理；selector、U04与context/response生产仍是独立必要规则。
