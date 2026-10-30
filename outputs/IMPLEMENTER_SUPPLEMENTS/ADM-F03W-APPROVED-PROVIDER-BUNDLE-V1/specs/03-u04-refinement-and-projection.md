# F03U Target refinement 与投影

U04与F03N的outer初始NNLS是两个阶段。本条款定义selector输出后的单次几何refinement、source weight、membership及当前消费的projection，不用outer的F32 correction替代这里的F64线性系统。

## F03U.1 调用条件和输入身份

当前前端仅当F03C返回CAPACITY_REACHED时装配refinement调用。CANDIDATES_EXHAUSTED和无candidate时直接沿用selected的初始XYZ/attributes，不强行补满15个target或复制点。selected_count恰好15但selector终止状态为EXHAUSTED时，同样不调用当前refinement路径。

source输入为F03G原始递增candidate_ids[N]及对应positions[N][3]、attributes[N]、原始feature[24][N]，N1..48。feature采用selector调用前的完整数据，不是衰减后/压缩后的矩阵。target输入按selector的选择顺序，初始XYZ和attribute来自selected global ID对应的同代geometry。全输入同callback generation。

F32坐标/feature精确转换为F64时，先保留原F32舍入值；不从原ADM更高精度重新取值。源ID、candidate ordinal、selected ordinal和owner slot是不同域。当前调试运输格式的局部target标签不能当作global source ID；本产品接口按选择顺序保留完整ID/attribute映射，不依赖研究fixture文件格式。

## F03U.2 去重与本路线边界

按target顺序保留第一个点。后续点与任一已保留点attribute相同且F64平方距离<=tau时为duplicate；tau=F64(R32(1/62)⊗R32(1/62))。XYZ减法/平方/求和依X/Y/Z顺序分别F64舍入。attribute不同不去重。

当前消费projection的路线要求refinement前后target数量与顺序一一对应。去重改变数量时拒绝U04_TARGET_DOMAIN_CHANGED，不把缩短数组继续关联原selected IDs。F03FD已提供F32/F64门限跨越和15→14实测拒绝向量，证明该分支可由自造有限坐标触发；选定受限实现的明确行为就是fail-stop/reset，不支持这类通用ADM输入。不得把当前歌曲未触发写成不可达证明或静默compact IDs。

## F03U.3 每source几何系统

用去重后固定的初始target t_i和本source s，全部F64。target位置不在source循环中更新。dot/sum按XYZ递增，禁止FMA。设q=Σs²、p_i=Σ(t_i×s)、G_ij=Σ(t_i×t_j)，B_ij=((q+1)+G_ij)-2p_j，L_ij=B_ji。

distance_i=Σ(s-t_i)²。L对角写为((attribute_i!=source_attribute?R64(0.1):0)+distance_i×R64(0.01))+old L_ii。这里比较完整U32 attribute，不是F03S的scalar/index state比较。H_ij=L_ij+L_ji，rhs每项-2。

F64 lower Cholesky按row递增、column0..row递增：sum从正零按k<column累加lower[row,k]×lower[column,k]。对角residual=H[row,row]-sum，正数取sqrt64，否则置正零；非对角为(H[row,column]-sum)/lower[column,column]。

forward按row递增，逐column<row减对应乘积，再除对角；backward按row递减、column>row递增，逐项减乘积再除对角。全部操作独立F64。signed solution先保留用于诊断，gain_i=R32(max(-solution_i,0))。非finite或零对角拒绝；不加入未定义ridge。F03FD给出exact-real SPD条件及其不等于F64舍入安全的范围；受限实现对数值退化明确fail-stop，不宣称通用坐标域支持。

## F03U.4 Source weight及投影矩阵

source_weight_j从正零按24个band递增累加pow32(F[b][j],0.2)，pow32遵循F03C。membership_gain G[j][i]为上节每source的gain，均finite非负F32，shape为N×T，不是T×N。

epsilon=R32(2.22e-16)，位模式257ff2d6。它不是2^-23，也不是当前callback测出的值。对每source j：

```
gain_sum = 按target顺序F32累加G[j][i]
gain_scale = 1 ⊘ (gain_sum ⊕ epsilon)
normalized = gain_scale ⊗ G[j][i]
weighted = normalized ⊗ source_weight[j]
M[i][j] = weighted ⊗ weighted
```

之后对每target row i，按source递增F32求row_sum，再求row_scale=1⊘(row_sum⊕epsilon)，每M[i][j]=row_scale⊗M[i][j]。两层各先算一个reciprocal再逐项乘，不能把每项直接除分母或交换两层归一化轴。

## F03U.5 实际消费的XYZ

每target三个累加器从正零，按source j递增、axis X/Y/Z递增求 projected[i][axis]=projected⊕(M[i][j]⊗source_position[j][axis])。target attribute低bit为1时，最终采用原initial target XYZ，保留原F32位模式。

当前前端将这里的F32 projection交F03G target installation，再生成F05H fit record。此处不提前Q15量化。另一诊断centroid及其Q15写回不是此接口的替代输入；下游panner或OAMD各自的量化仍遵守各自条款。

返回selected顺序的projected XYZ、原IDs/attributes、source_weights[N]、membership_gain[N][T]及规范化M[T][N]诊断。内部字段叫position时仍是当前模型的XYZ容器，不按旧研究结构的角度/距离字段名重新解释单位。

## F03U.6 状态、失败与验收

一次调用只做一轮几何solve和projection，不反复更新anchor求收敛。矩阵、gain和membership为callback局部scratch；owner历史与提交由F03G/F05H负责。skip时不虚构source_weights或gain缓存。

验收包括source/target轴顺序、原始feature与residual feature区分、attribute对角penalty、F64系统、两层reciprocal、零gain/weight、protected anchor、原始投影与Q15边界、数量变化拒绝。完整feature/geometry生成、所有admission/duplicate/degenerate可达性及真实ADM空间效果仍未闭合，本条款不能单独证明整条refinement生产链通过。
