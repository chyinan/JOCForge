# F04P Component共享参数state

本文件定义15路component在每64样本block的共享peak/attenuation/envelope更新。不是每row独立的limiter；F04A.8的filter读取本次next gain和旧output level。当前8项固定参数由F04PG生成并核验身份；其他profile、完整controller及真实ADM验收不由参数齐备推定。

## F04P.1 输入、参数与状态

输入15×64 finite F32、共享旧state及只读参数。参数为c[5] finite F32、reciprocal_pivot F32、target_pivot F32、envelope_weight F32；以下使用范围要求c0～c4/envelope_weight在[0,1]、reciprocal_pivot在[0.5,1)、target_pivot在[0,1]。D11-F已有F04PG的固定数学生成路线与8词身份核验，原研究表不复制；合成参数用例与该固定数据分开。

可变 state 的规范字段为 peak、previous_gain、envelope、smoothed_gain、target_gain、output_level（均为 F32），以及 hold、blocks_processed（非负整数）。peak 保存上一块的全局峰值；正文的 previous_peak 是 old.peak 的语义别名，序列化只接受 peak。缺失字段、额外 previous_peak 字段或重复字段均拒绝，不从 expected 补全。当前 profile cold 为 peak=0、previous_gain=1、hold=0、envelope=1、smoothed_gain=1、target_gain=1、output_level=2^-8、blocks_processed=0。

只读 profile 与可变 state 分开安装。gain_scale=2^-8、clamp_limit=1、hold_limit=1、processing_format=64、row_count=15、block_sample_count=64。它们在初始化和 reset 后保持不变，不属于 old/next state。参数词投影还包含三个固定 U32 诊断标签：active=1、direct_route_flag=0、filter_iteration_count=1。这三个标签表示本验收接口选定的启用 component、非 direct 路由和单次块调用；由 profile 初始化安装，不从 PCM 或峰值推导，不控制 F04P 分支。filter_iteration_count 不表示 callback 的 24 块，也不表示 reciprocal 的 4 次修正。

before/after 词投影均从各自对应 state 产生。peak_before_f32_u32 对应 peak（after 中该历史字段名也保持不变）；previous_gain_f32_u32、envelope_f32_u32、smoothed_gain_f32_u32、target_gain_f32_u32、output_level_f32_u32 分别为同名 F32 位模式；gain_hold_counter_u32 对应 hold。gain_hold_limit_u32、clamp_limit_f32_u32、gain_scale_f32_u32、row_count_u32、block_sample_count_u32、processing_format_u32 和上述三个标签来自只读 profile。before 输出全部 16 个词；after 只输出 peak、previous_gain、hold、hold_limit、envelope、smoothed_gain、target_gain、output_level 对应的 8 个词。blocks_processed 位于完整 state，词投影不包含它。

状态scalar必须有限；selected正常state的gain/envelope/peak非负，hold<=2^31-1。参数缺失返回PARAMETER_UNAVAILABLE，不代入测试值。跨callback复用同一state；reset恢复完整cold，不只清peak。

## F04P.2 Peak与reciprocal attenuation

current_peak从F32正零开始，按row0～14、sample0～63扫描abs(input)。仅严格大于旧峰才替换；witness因此为最早达到全局最大值的位置，全部零时null。scaled_peak=max(current_peak,old.previous_peak)⊗old.envelope。

scaled_peak<=1时attenuation=1。scaled_peak>1时令n=scaled_peak、e=0；按顺序反复n=n⊗0.5、e=e-1，直到n<1。该分支n最终属于[0.5,1)，要求1<=e+128<=254，否则失败。

n<=reciprocal_pivot时r=(1⊖n)⊗2，否则r=1⊖(0.5⊗n)。恰好执行4次修正：p=r⊗n，d=0.5⊖p，correction=(d⊗r)⊗2，r=r⊕correction。不能以普通F32除法替代这条所选运算网络。

attenuation=(1⊗r)⊗R32(2^(e+1))。最后因子按binary32 normal exponent e+128构造，禁止把e当IEEE bias后的实指数。所有⊕/⊖/⊗依N00分别舍入，无FMA。

## F04P.3 Target、hold与smoothed gain

先保存new.previous_gain=old.smoothed_gain；maximum=old.envelope⊗attenuation。working hold/target/smoothed分别从旧对应state取值。

若smoothed>maximum：target=attenuation、hold=0、smoothed=maximum。

否则若hold<1：hold加1，target/smoothed保持。

否则：target=(attenuation⊖(c4⊗attenuation))⊕(c4⊗old.target_gain)；candidate=target⊗old.envelope。candidate>smoothed时替换smoothed；再在smoothed>maximum时把它限制为maximum。比较均严格大于；不能把等号引入落增益分支。

## F04P.4 Envelope与提交

使用F04P.3更新后的target与旧envelope。target<target_pivot时，term=(((target⊗old.envelope)⊗envelope_weight)⊗2)⊗c1，new.envelope=term⊕(old.envelope⊗c0)。否则new.envelope=(old.envelope⊗c2)⊕c3。相等进入后一分支。

完整next state为current_peak、已保存的new.previous_gain、working hold、new.envelope、working smoothed/target、output_level=2^-8、blocks_processed+1。每block提交一次；envelope变化供下一block的scaled_peak，不反馈到本block maximum重算。

本block供filter的next previous_gain/smoothed_gain来自上述next state，而output_level/gain_scale/clamp来自block开始state/固定profile。F04A的24 blocks按递增顺序消费；全15路共同影响peak和参数，不为inactive零row跳过时钟。

## F04P.5 错误与验收

shape/type/参数范围/非有限输入或中间结果、normal exponent构造范围失败时不产生可用next state，programme按I03失败。原生异常中间状态不作保证。只支持48k、15row、64格式的当前profile；其他constructor参数拒绝。

合成参数测试覆盖under-limit hold增长、target recurrence、下降增益、previous peak的一block记忆、两个envelope分支及连续terminal。固定生成数据另有23项完整before/after state用例，见F04PG；预期state不是运行时seed。独立审查、全数值域/错误提交及整链验收仍未完成。
