# F03U/F03S 数值与身份边界审计草稿

此草稿只定义现有接口和已验证边界，不将拒绝边界改称完整通用 ADM 行为。所有案例是自造数值，没有原始音频、秘密值或编译几何表。完整 programme 空间效果验收未运行。

## 1. Selector 与 refinement 的去重判定不是同一数值运算

令 `tau=R32(R32(1/62)*R32(1/62))`，其 F32 words 为 `39886429`。Selector 对坐标差、平方和逐项 F32 舍入，要求 `distance32>tau` 才保留同 attribute 候选。Refinement 将已有 F32 坐标精确转换成 F64，按 X/Y/Z 的减、平方、累加分别 F64 舍入，按 `distance64<=F64(tau)` 去重。

自造相同 attribute `2816` 的两点，XYZ F32 words 分别为 `[00000000,00000000,00000000]` 和 `[3c78da6d,3bb07d05,3a2d5454]`。Selector 距离为 `3988642a`，因此该距离过滤不删除第二点。Refinement 距离为 `0.00026014565984281806`，小于 `F64(tau)=0.0002601456653792411`，因此删除第二点。坐标在单位盒内。该案例否定“Selector 的距离门限保证 Refinement 不会去重”的无条件证明；尚未证明该对点一定被完整 Selector 同时选中且填满 capacity。

去重独立行为是：按原 selected 顺序，对每个候选，只要任一已保留点与它 attribute 全 U32 相等且 distance64<=tau，就跳过该候选；否则原点记录完整保留。不能只比较上一点。Identity 搬运应按 survivors 的原 selected ordinal；global ID、初始 ordinal、attribute、XYZ、owner slot 是不同域。

构造 15-target capacity 调用：头两点如上，其余 13 点见向量。冻结研究数值程序实际产出 14 个 target，而消费 projection 连接拒绝 count 改变。因而现有连接并不包含 15→14 的 compact identity/owner 提交路径。冻结程序内部去重保留完整 anchor，但后续连接仍使用原 selected IDs，并要求 attribute 数组与原数组逐项相同。不能把该内部保留行为误称整个 caller 已完成 remap。

当前可实施失败合同：发现 survivor_count != selected_count 时，没有正常 projection/target-owner 提交；终止当前 programme，按 I03 丢弃状态并完整 reset。该合同只是现有受限实现边界。泛型 ADM 支持仍需完成 survivors identity 映射、projection 的 N×survivor_count 维度、owner 安装与 metadata/outer 同代提交，再验证真实空间效果；不得静默删除 ID、补零 gain 或拿原 IDs 配压缩矩阵。

## 2. 几何系统的 exact-real 条件

设 `u_i=t_i-s`，并令非负对角 penalty 为 `d_i=0.2*attribute_mismatch_i+0.02*||u_i||²`（F03S 将 attribute mismatch 换成 scalar/index-state mismatch）。在不考虑浮点舍入的实数计算中，现有对称几何系统可重写为：

`H = 2*11ᵀ + 2*U*Uᵀ + diag(d)`。

对任意非零向量 z，`zᵀHz=2*(Σz_i)²+2*||Σz_i*u_i||²+Σd_i*z_i²`。因此 H 半正定；它正定的充要条件是不存在非零 z 同时满足：仅在 d_i=0 的 indices 支持、Σz_i=0、Σz_i*u_i=0。若所有 d_i>0，则正定；若只有一个 d_i=0，其余均正，也正定，因为 Σz_i=0 强迫唯一剩余 z 为零。

d_i=0 当且仅当该点等于 source 且 mismatch 为零。两个或更多相同 source 位置且 mismatch 为零的 target 给出奇异系统，例如 z=(1,-1,0...)。F03U 同 attribute 的相同点会被去重；不同 target attribute 至多有一组与 source attribute 相等，因而在 exact-real 数学中，完成去重的 F03U 几何系统正定。这是数学结构证明，不是 F64 Cholesky 任意有限输入的舍入安全证明。距离很小、数值界限、累计误差及 condition bound 仍须另证。

F03S 的 index state 全零、scalar flag 零并不提供 attribute penalty；Refinement 后的 XYZ 也没有已证两点 separation。不能将 F03U 的初始点去重证明转用于 F03S。单位盒内全零 source 与重复零 target 的实际冻结 F32 消费：n=1、n=2 返回有限结果，n=3、n=15 报奇异 correction。n=2 接受是 sqrt32/累加残差形成的数值结果，不能反证 exact-real 奇异或当成恢复算法。

当前异常合同仍是无正常系数输出、按 I03 全 programme fail/reset；禁止临时 ridge、pseudoinverse、对角加 epsilon、source 零填充或沿用上一代系数。选定域可达性仍须 upstream producer 或 native 专项规则证据闭合。

## 3. Active energy admission 不足以在 F32 中证明 D 非负

实数理想参数满足 `initial_scale²*energy_ratio=1`；在 active 条件 `Ey<=Eb*energy_ratio` 下，scaled energy Ei<=Eb，使 `D=(2cross)²-4*(Ei-Eb)*Ev>=0`。现有 F32 admission、矩阵乘和逐项能量累加与这个理想恒等式不是同一运算。

向量给出非负 finite context[15][2]、非负 response[15]、seed[2]、correction[2]，按现有 admission 得到 `Ey=threshold=3f21866c`、`Eb=3f800000`，active=true。逐项 F32 scale 后 `Ei=3f800001`、cross=0、Ev=1，实际 `D=-2^-21`，冻结模型拒绝 negative discriminant。这个案例证明接口及 active admission 边界可达，不证明这些 seed/correction 由实际 NNLS 与实际几何联合生成；向量明确标出该区别。

没有证据支持把负 D clamp 到零视为既有行为。完整闭合需要：从真实 response/context 几何域及 NNLS 输出证明充分数值界限，或恢复所需边界行为并验证；不能引用某首歌曲没触发。

## 4. 精确剩余项

1. 去重 count 变化：已确认当前模型拒绝及可构造阈值 crossing；尚缺同代 survivors→global-ID/owner 全 caller 合同与消费验证。
2. F03U exact-real SPD：已给证明；尚缺所需坐标、F64 条件数/舍入界及所有 admitted 数据的 finite 求解保证。
3. F03S singular：已有 frozen Python 明确失败和单位盒 n3/n15 输入；尚缺 target projection producer 的不可达证明或真正处理规则。
4. Negative D：现有 active admission 的 F32 反例已完整构造；尚缺 NNLS+geometry 联合可达性及必须处理域。

以上结果不得将整体 IMPLEMENTATION_READY 标为 YES。
