# F04PG Component共享参数的固定数学生成

本条款提供F04P的5个系数和3个阈值/权重。范围为48000 Hz、每block64样本、全15路共享参数；不是每row的独立limiter。参数表达由恢复行为重建，不推定原设计使用了下面的半衰期或dB说明。仅内部研究和内部实现，不公开或第三方分发。

## F04PG.1 Q15与系数

定义 Q15_RNE(x)=RNE_integer(32768x)/32768。整数最近偶数；结果为exact binary32有理数。dt=64/48000秒。

c0=Q15_RNE(2^(−dt/(1/25)))，c1=1−c0；c2=Q15_RNE(2^(−dt/(4/25)))，c3=1−c2；c4=Q15_RNE(2^(−dt/(1/200)))。这组等值设置对应40/160/5ms半衰期参数化，不使用原研究5项表生成。

依次Q15整数为32020、748、32579、189、27238；两组互补和为32768。c1/c3由已量化c0/c2的补数生成，不分别量化同一个近似函数。

## F04PG.2 阈值与envelope weight

reciprocal_pivot=ceil(32768/sqrt(2))/32768，是向上Q15网格；不能误用最近偶数。target_pivot=Q15_RNE(10^(−3/40))，envelope_weight=Q15_RNE((1/2)×10^(3/40))，对应固定1.5dB幅度比例的等值表达。

三项Q15整数为23171、27571、19472。target_pivot和envelope_weight分别从精确公式量化，不能从已量化target_pivot取倒数生成weight。比较方向、等号及四次reciprocal更新仍按F04P，不以本公式替换其运行网络。

## F04PG.3 数据、身份与更新

数据见 `../parameters/component-parameter-mathematical-v1.json`。100/200位精度对固定公式的整数舍入一致；8个F32词与选定消费参数相同。原研究表、源码和地址未复制。运行可读取生成固定词，也可重生成后与整数/词身份核验；不依赖某平台pow/sqrt恰好给相同中间结果。

该数据在programme初始化安装并冻结。cold和reset采用F04P.1完整state；每个64样本block按F04P推进一次，24个block/callback，包含inactive的零row。输出level/gain读前后代的顺序仍按F04A.8，不能因参数已生成而改成所有字段取next state。

## F04PG.4 等价核验和限制

`../test-vectors/COMPONENT-PARAMETER-GENERATION-VERIFICATION.json` 给出完整before/after词及state，包括连续静音/单位峰/超限/一block previous-peak、hold/恢复、target threshold相等与下邻、reciprocal pivot相等与上邻。原模型和生成参数绑定下全部结果一致。

等价仅为当前8参数身份和同输入同状态的F04P转移；固定参数相同且运算/分支不变，不靠近似容差判通过。它不关闭全controller、其他sample rate/row/block配置、真实ADM新计算或全链失败提交。

独立隔离审查 r2 已确认三项接口问题关闭，23 块和全部已声明词投影一致。范围见 test-vectors/COMPONENT-PARAMETER-LOCAL-REVIEW-STATUS.json。原研究参数的历史身份对照是研究侧证据，不让实现者读取原表；本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。

## F04PG.5 验证报告 v2 的执行契约

本报告使用独立的验证 schema，不是 U00 的逐例冷启动请求。execution_plan 顺序必须与 cases 顺序逐项一致；case id 唯一。每个 sequence 从完整 cold 开始，按 case_ids 推进。每项 execution 明确 sequence_id、block_index（从 0 开始）、predecessor 和 reset_before。第一项 predecessor=null、reset_before=true；后续 predecessor 为上一项 id、reset_before=false。成员缺失、乱序、前驱不符或中途 reset 均拒绝。

continuous_peak_recovery 含 continuous_0～18 的 19 个有序块。只在第 0 块冷启动，后续使用实际计算的 next state。输入先构造 15×64 正零；max_magnitude 非零时，在 negative_witness 位置写负峰，在 positive_equal_witness 位置写等幅正峰。其余样本保持正零。峰值为零时不写入负零，witness 为 null。

其余四个 sequence 各含一个块，互相独立冷启动。target_pivot_equal/below 在 cold 后用 input.state 的完整八字段 state 替换，PCM 全正零。reciprocal_pivot_equal/above 从 cold 开始，只在 peak_row_sample 写入 peak_value，其余样本正零。state override 只允许 sequence 第一块；不得从 expected_before_words 或 expected_state 构造初始状态。

验收 runner 的映射只接收 id、execution、input 和 execution_plan，生成重置、前驱及 PCM/state 配方。运行算法后才读取 expected 比较。expected_before_words、expected_after_words、expected_state 均为断言。固定诊断标签与词投影按 F04P.1 从 profile/state 构造。映射验证覆盖乱序、错误前驱、中途 reset、连续块 state override 四种拒绝；23 块输入驱动重放的完整 next state 与期望一致。
