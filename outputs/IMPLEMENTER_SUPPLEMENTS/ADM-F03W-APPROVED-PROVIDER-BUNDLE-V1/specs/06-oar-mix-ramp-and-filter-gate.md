# F06M OAR gain ramp、混合与关闭的 per-task filter

本条款定义实际消费的六路 gain ramp/mixer 和 task sum。点目标由独立 panner/metadata producer 提供；本文件不能替代该 producer，也不把 per-task filter 的未执行代码当作数值算法。

## F06M.1 选定 filter gate

选定 OAR adapter 构造完成配置后，明确把 per_task_output_filter_enabled 写成 false。task initializer 因该配置建立“没有 filter”的状态；处理器仅在 filter 实例存在时调用其处理函数。因此本 profile 将 mixer 输出直接作为每 task 输出，没有 per-task attenuation/envelope/audio-history 算法，也不增加它的 32 样本延迟。

这条规则来自配置 writer、initializer 和调用 guard 的连续静态绑定，并与已完成的 OAR 执行记录一致。导入但没有调用的旧 gain-history/direct-division helper 不是该运行链的消费规则。旧模型诊断中的 peak=0/attenuation=1 只是关闭路径的记录标签，不作为已计算的峰值或增益结果交付。实现不能运行该 helper，再用类似标签冒充关闭路径。

shared post-OAR DV32 仍由 F06D 执行；关闭 per-task filter 不关闭它。其他启用 per-task filter 的配置不在选定 profile 内，必须拒绝该配置，不能套用本关闭规则作完整复现。

## F06M.2 数据、参数和 gain-row state

mix32 接口输入 N 个 gain rows 和对应 N×32 个 finite F32 source 样本，N=0～128。每个 gain row 有 current、target、step（各 6 个 finite F32）和 remaining（6 个非负整数）。输出为 6×32 F32。空 N 的输出为正零，next rows/proof 为空。

每个源各自拥有一行 state，cold 为 current/target/step 全正零、remaining 全 0。安装 target 接口要求完整六项，逐项转 F32 后检查有限；它只替换 target，其余字段保留。不得从新 target 直接覆盖 current 或清 step。

只读 ramp[i]=R32(i/32)，i=0～31；最后一项是 31/32。epsilon=R32(1/10^6)，delta_gate=R32(1/10^4)。update_flag 为整数 0 或 1，interpolation_blocks 为整数 0～2048。类型、shape、非有限或 F32 转换溢出时按 I03 拒绝；原生域外错误状态不推定。本接口不接收外部未知 ramp 表。

每块输出先为正零。按 source 0～N−1、channel 0～5、sample 0～31 的顺序消费，所有 N00 运算分别 R32，不融合 multiply-add。

## F06M.3 Gain 分支和时钟

每个 source/channel 的 previous=current[channel]。abs(target)<epsilon 且 abs(previous)<epsilon 时，整 channel 跳过：输出不增加，current/step/remaining 保留；已安装 target 仍保留。两个严格小于均成立才跳过，等于 epsilon 不跳过。

difference=target⊖previous。abs(difference)≥delta_gate 时，仅 update_flag=1 才将 remaining=interpolation_blocks；interpolation_blocks≥1 时另设 step=difference⊗R32(1/interpolation_blocks)。除法单独 R32。interpolation_blocks=0 时不计算倒数，也不覆盖旧 step。update_flag=0 时 remaining/step 保留。

abs(difference)<delta_gate 时，无论 update_flag，remaining=0、current=target。等于 delta_gate 走前一分支。

上述判断后 remaining>0 时，每 sample gain=(ramp[sample]⊗step)⊕current，output[channel][sample]=(gain⊗PCM[sample])⊕旧 output。整块完成后 current=current⊕step，remaining 减 1。即使最后一块 remaining 从 1 变 0，也不额外把 current 设成 target。

remaining=0 时，每 sample output=(target⊗PCM)⊕旧 output，块后 current=target。输出没有 clip、peak 或 attenuation。next row 保留 target、更新后的 current/step/remaining。零 source 样本仍推进非跳过 channel 的 gain 时钟。

## F06M.4 Callback 和 task sum 的消费顺序

已核验的 OAR 运行采用 16 个逻辑源、8 个有序 task，每 task 两个连续源：[0,2)、[2,4) 直至 [14,16)。这描述该执行的逻辑划分，不把物理线程先后作为数值顺序。通用 host-dependent task factory 和完整 metadata producer 的绑定仍另行审计，不能从这一次划分推定所有 CPU 配置。

该执行每 callback 有 1536 样本，按 48 个 32 样本块递增消费。每 callback 开始，为 task 的两 gain rows 安装本次六路 targets；只有 canonical metadata records 与上次缓存不同且 block=0 时 update_flag=1，其余为 0；interpolation_blocks=48。首次 records cache 为空，所以 changed=true。全部 48 块成功后提交 rows 和本次 records cache。

F06OM、F06OP 和 F06OE 已定义规范记录字段、点目标及完整 60 字节比较原语；实际 queue snapshot、header-change、source count/active 条件及 callback 提交仍需绑定。不能用 targets 数值相同替代 records 的全字节相同，也不能将未绑定 writer 的保留字节直接清零。本条款的 standalone mix32 用例只给定 update flag，不声称已经验证完整 producer。

每 task 六路输出块先拼成 6×1536；per-task filter 按 F06M.1 关闭。task sum 从 6×1536 正零开始，按 task 索引递增、row 索引递增、sample 索引递增执行 sum=旧 sum⊕task_output。不得改成把全部 16 源一次顺序相加，或依线程完成先后归约。task sum 后交 F06D，不在此引入额外 filter。

## F06M.5 向量和完整 gain state 摘要

OAR-MIX32-VERIFICATION.json 的 cases 每项独立从 N 个 cold gain rows 开始，每次到达 reset_before_blocks 中的索引时重新 cold；随后应用该项明确的 initial_row_overrides（只覆盖列出的同名数组），再安装 input.initial_targets。之后按 input.blocks 的有序块执行。target_updates 以十进制 block_index 字符串为键，在该块运算前安装完整的新 targets；顺序是 reset/overrides/初始 target、当前块 target 更新、source 生成、mix。每块每 source 的 PCM 由 source_rule 生成；update_flag 为 block_index 是否属于 update_blocks。interpolation 取 input.interpolation。所有 input 仅用作配方，不读取 expected 状态作 seed。

GAIN_ROWS_STATE_V1 布局为 8 字节 ASCII OMIXROW1，接 N U32；每个 source 依次为 current[6]、target[6]、step[6] 各 F32，再 remaining[6] U32，全部小端。长度为 12+96N 字节。before_state_sha256、expected/actual_state_sha256 分别是计算前和完整 next state 的 SHA-256。摘要布局对 remaining 使用 U32；超过布局范围时拒绝摘要，不推定原生无限时钟行为。

expected/actual_rows_after 是完整四数组 state。expected/actual_output_sha256 是六路按 row/sample 顺序连接的 F32LE。proof 按实际非跳过 source/channel 顺序追加，每项有 source、channel、branch 和 remaining_after；branch 为 INTERPOLATED 或 CONSTANT_MULTIPLY_ADD，跳过没有条目。

group_sum_case 从 16 个 cold rows 安装 target_rule，并按 source_rule 生成 32 样本，依 ordered_task_ranges 分别混合，最后按 F06M.4 task sum。它的 output 摘要为完整 6×32 F32LE。当前 12 组共 80 块、15360 个输出词，以及分层 sum 均与实际模型一致。无继承研究上下文的局部审查独立重算通过，857 项检查、零新增问题；审查范围及限制见 OAR-MIX-GEOMETRY-LOCAL-REVIEW-STATUS.json。该审查没有核验原生 filter 构造证据；完整 renderer/metadata/真实 ADM 仍未闭合。
