# F03S Outer 能量、correction 与最终矩阵

输入context A[15][n]、response b[15]、F03N初始seed x[n]、active target XYZ[n][3]及本source XYZ[3]，n0..15。当前一次callback逐source使用新局部state，输出n×48 F32矩阵；inactive target slot不补成求解变量。

## F03S.1 接口、顺序和数学参数

source call_order为0..47的唯一有序子集，uncalled_sources必须恰为其补集。context为row-major15×n，response每实际调用source提供完整15项；target XYZ按active destination ordinal。输出matrix先全部正零，按call_order逐source计算，将n项最终系数按ordinal写入其source column；未调用source保留零。

n=0直接输出空矩阵、无solver调用，不补capacity15零变量。每source的correction index_state[n]初始化U32零，scalar_flag当前0；不沿用上一source或上一callback的index state。A、b、XYZ、seed均finite F32。

当前数学参数：energy_ratio=R32(10^-0.2)，initial_scale=R32(10^0.1)，epsilon=2^-23，floor=R32(10^-6)，dominance_ratio=R32(0.99997)，flag_penalty=R32(0.1)，distance_penalty=R32(0.01)。前两项与原接受路线的F32位模式分别为3f21866c/3fa12478，dominance为3f7ffe09；它们由公式提供，不需编译数组。不得替换成近似的十进制截断值。

定义dot32(u,v)从正零按索引递增执行 total=total⊕(u[i]⊗v[i])；energy32(v)=dot32(v,v)，matvec32按每row执行相同dot。F32 sqrt为R32(sqrt(输入F32的精确数值))，不把能量换成F64归约。NNLS内部精度仍按F03N。

## F03S.2 能量分支和fallback

y=matvec32(A,x)，Eb=energy32(b)，Ey=energy32(y)。Ey<=Eb⊗energy_ratio时进入active correction，等号属于active；否则fallback。

fallback scale=sqrt32((Eb⊕epsilon)⊘(Ey⊕epsilon))，各seed乘scale得到scaled。随后执行F03S.5的floor/prune。原接受路线在outer尾部再执行同一floor/prune；所选条款保留此调用顺序。

## F03S.3 几何correction线性系统

设source s，target t_i，均按XYZ三轴顺序做dot32。q=energy32(s)，p_i=dot32(t_i,s)，G_ij=dot32(t_i,t_j)，distance_i=energy32(s-t_i)，三个减法分别F32。

先令 B_ij=((q⊕1)⊕G_ij)⊖(p_j⊗2)，L_ij=B_ji。每个对角增加 delta_i=((scalar_flag!=index_state[i])?1:0)⊗flag_penalty ⊕ distance_i⊗distance_penalty，按此两项相加后再加到原对角。构造对称系统 H_ij=L_ij⊕L_ji，rhs每项-2。

使用F32 lower Cholesky：对row i递增、column j<i递增，v=H_ij⊖Σ_{k=0..j-1}(C_ik⊗C_jk)，sum从正零按k递增；C_ij=v⊘C_jj。row_square_sum按j顺序累加C_ij²。r=H_ii⊖row_square_sum，r>0时C_ii=sqrt32(r)，否则C_ii=正零。

forward按i递增，从rhs_i开始逐j<i减C_ij⊗forward_j，再除C_ii；backward按i递减，从forward_i开始逐j>i递增减C_ji⊗solution_j，再除C_ii。每减/乘/除独立F32。correction_i=R32(max(0,-solution_i))。

零对角、非finite或不能完成的singular系统拒绝。F03FD给出n=3/15重复零点的实际拒绝向量；受限实现的规则是无正常输出、按I03全pipeline fail/reset。不能加epsilon或任意pseudo-inverse后声称原路线完整；这类输入明确在支持域外，不宣称通用ADM能力。

## F03S.4 Active二次能量更新

先保存unscaled y=A x；scaled_seed=x⊗initial_scale，scaled_y=y⊗initial_scale。v=A correction。cross=dot32(v,scaled_y)，c=cross⊕cross，Ev=dot32(v,v)，Ei=dot32(scaled_y,scaled_y)，Eb=dot32(b,b)。

D=(c⊗c)⊖((Ei⊖Eb)⊗(Ev⊗4))。D<0拒绝，不clamp为0。F03FD给出满足active admission等号却产生D=-2^-21的接口级反例；该反例不证明NNLS+geometry联合生产可达，但足以要求实现保留拒绝分支，不能从admission推导无条件D非负。step=(sqrt32(D)⊖c)⊘((Ev⊕Ev)⊕epsilon)。output_i=(step⊗correction_i)⊕scaled_seed_i。系数后处理前不提前clip到[0,1]。

input b有15项、seed/correction有n项。不能复用旧square helper要求三者长度相同的检查，或把XYZ表扩到15行掩盖n<15。

## F03S.5 Floor与dominance

floor时仅当floor>coefficient将该项置正零，相等保留。然后从正零按ordinal递增F32求total；maximum按ordinal选择最大值，同值保留最早ordinal。

maximum<total⊗dominance_ratio时保留全部系数。否则输出全正零，仅最早maximum位置写R32(min(total,1))。相等进入one-hot分支，不是保留原向量。n>=1才能执行此步骤；空向量走F03S.1。

## F03S.6 提交、验收与未完成项

每source的NNLS/correction局部buffer在调用后丢弃，不是audio/history checkpoint。当前outer消费F03N输出seed，包括ITERATION_LIMIT状态的当前seed；诊断必须保留该状态，不能标为SOLVED。非finite、singular和negative discriminant按I03停止，不跳过source填零继续。

验收覆盖n0/1/rectangular/15、source未调用列零、energy等号、epsilon、距离/flag对角penalty、Cholesky、active更新、floor相等和dominance相等/同max。完整context/response、selector/U04和空间几何生产仍未关闭；局部数学参数完整不表示front整体可实施或空间听感已通过。
