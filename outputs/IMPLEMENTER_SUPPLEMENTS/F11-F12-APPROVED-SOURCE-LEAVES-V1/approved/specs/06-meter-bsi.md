# F06 测量、BSI 和 DRC

状态：`PARTIAL`。F06L 定义 log/peak/energy 原语；F06I 定义 integrated gate 持续状态；F06W 定义逻辑输出与窗口时钟。F06KG已提供完整K矩阵数学规则并通过局部r2复审；F06TP已交付用户授权的11个非秘密FIR值及完整cascade状态/指标，9例局部r1复审通过。F06OC的模型OAR callback/commit合同已通过局部复审，F06OF定义frontend输入/时钟连接。F06ND selected实际参数与连续统计已局部复审通过，F06WH meter-finalize→none-DRC组合也已通过r4局部复审；Q07仍开放的是实际ADM meter frontend、真实programme执行、三次zero drain、finalize同值绑定及whole gate。局部通过不能替代全meter；选择器、dialnorm和固定profile字段关系如下。

## F06.1 测量通路

meter 从 ADM 和独立状态冷启动，其空间渲染和 DV32 延迟与主音频分开。最终 measurement 不能读取主通路的音频状态、已有测量结果或参考元数据作为预测输入。

公开测量算法允许参考 ITU-R BS.1770-4；本 profile 指定该版本。选定OAR渲染局部合同见F06OC/F06OP/F06M，共同窗口前缀/裁剪见F06W，true-peak精确FIR参数与运算顺序见F06TP；这些已交付规则不认证上游完整渲染或整个meter组合。引用标准不能补齐仍未定义的依赖。

## F06.2 loudness 选择

输入为已具有 F32 精度的 dialogue LUFS、ITU LUFS、ungated LUFS、speech 百分比和阈值，各项均有限。普通 speech 百分比和阈值必须在 [0,100]。选定 DI 关闭模式另允许 speech=−1 哨兵，仅能由下文 getter 规则产生，不能把任意负百分比视为有效。各项先除以 128 并 R32。

若 threshold>=percentage 或 dialogue<=-0.46875，使用 fallback：ITU<=-0.9375 时取 ungated，否则取 ITU。其他情况取 dialogue。选中的值乘 128 并 R32，得到 selected LUFS。

内部 gating 枚举为 ungated/level/dialogue=0/1/2；raw meter 对应 0/2/1，wrapper 对应 1/3/2，公开主测量结果对应 0/2/1。枚举转换需要保留，不能把不同域中相同数字当作相同语义。

本 profile 阈值为 100，因此合法 speech 百分比均进入 fallback；dialogue_intelligence=false 的 producer 仍需按 F06.1 定义，不得以歌曲固定 LUFS替代。

### F06.2b DI 关闭的 getter producer

选定配置要求 metering_mode=1770-4、dialogue_intelligence=false、R32(speech_threshold)=100。wrapper 配置 mode=2，normalized meter mode=3。在本模式中跳过 speech classifier；dialogue internal=−15/16，host dialogue=(−15/16)⊗128=−120；speech internal=−1/128，host percentage=(−1/128)⊗128=−1。它们是模式专用 getter 哨兵，不是对输入声音的语音或静音分类。

ITU 必须来自本次 F06I/F06W 独立完整测量。归一化 threshold≥percentage，因此按 F06.2 进入 fallback。ITU internal>−15/16 时选择 ITU，internal gating kind=1、公开主测量 kind=2；否则使用 F06U 新计算的 ungated 值及 kind=0，不能借 dialogue/speech getter 哨兵替代它。其他 metering mode、DI=true 或不同 threshold 不属于当前 profile 请求，返回不支持配置。

METER-DI-OFF-GETTER-VERIFICATION.json 的每个正常 cases 项独立消费 input 中的 ITU_host 与配置，不使用期望结果初始化。输出投影包含 dialogue_word、speech_percentage_word、selected_word、internal_selected_word（四项均为 F32 U32 位模式），以及 internal_gating_kind、public_gating_kind、wrapper_config_mode、normalized_mode 和 speech_classifier_executed。词和枚举依本条款及 F06.2 计算。rejection_cases 表示三个不支持配置的入口拒绝。

该旧报告的 unavailable ITU 项记录旧研究 helper 没有 mean 输入时的限制。当前 F06U 及 METER-UNGATED-FALLBACK-VERIFICATION.json 已提供新 mean 与耦合 fallback；旧 helper 的拒绝不再作为当前规格的永久缺口。新实现仍须实际计算 mean，不得使用旧报告作为运行时预测输入。

## F06.2a Programme 测量绑定

先从同一完整 ADM/config，以独立冷 meter 状态完成整首测量和规定的有效样本裁剪/尾部处理，得到最终 dialogue、ITU、true-peak、gating kind 和 selected LUFS。随后生成 main programme 的全部 BSI/ID1 输入；使用这个最终结果，不使用 source 0/1/2 的滚动 loudness。

selected LUFS 只在此次完整测量后绑定为 programme 参数，dialnorm 对所有 source 相同；DRC 的 U16 控制仍逐 source/块生成。ID1 timer 接收同一 programme 的最终 dialogue/ITU/true-peak 及 gating kind。prefix 窗口也使用完整歌曲的这次新测量；不得把 prefix 截短测量后再用其 LUFS。

因此独立实现需要预遍历或等价的完整测量阶段，再从冷 main 状态计算编码。各阶段可以重新读取 ADM；只消费自己新计算的结果。测量算法本身及具体数值绑定向量仍为 Q07。

## F06.3 dialnorm

若 `abs(R32(selected_lufs⊖(-120)))<0.000001`，dialnorm=31。否则先求 `t=trunc(selected_lufs⊖0.5)`，再输出 `-sat[-31,-1](t)`，宽 5 位。向零截断不能替换为 round-to-nearest。

## F06.4 字段关系

loro 的 preferred mode 为 2；四个 -3 dB mix level 编码均为 4。mix metadata 存在。information metadata 存在，copyright=true、original=true、bitstream mode=0、source sample-rate-code=false。

surround EX mode按F01E解析的 `authored_source_mode_byte` 产生：host byte=3时编码2，否则0。DBMD section9的 `warm_trim_mode` 先转换为host byte=`warm_trim_mode+1`，两者不是同一域。当前warm trim3→host byte4→surround EX0；不能把trim mode3直接代入判断。

none 的 line/RF profile core code 均为 0。最终仍运行 DRC 控制 producer，当前 source frame 每帧 RF byte 为 255，六块 line byte 为 0。这是现有实际运行的检查断言；selected none producer局部规则/参数已通过复审，真实programme消费及其他配置行为仍属于Q07；实现者不能用断言替换 producer。

每帧提供同代 U16 DRC 控制输出，wire 采用高 8 位。BSI 的消费 source index 必须连续，从 0 开始，包括 priming 两帧。当前带宽 code=48，不从数据文件名推断。
