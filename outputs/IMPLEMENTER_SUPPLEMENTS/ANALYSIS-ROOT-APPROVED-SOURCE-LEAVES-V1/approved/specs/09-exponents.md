# F09 POWER 与 PRIMARY 指数

状态：`SELECTED_POLICY_PARAMETERS_SUPPLIED`。位模式转指数、刷新误差映射及序列化分组的规则已定义。D06-E的32项映射由F09.6完整生成；普通刷新threshold、4个位置权重和D06-N/D07必要标量由 `parameters/policy-authorized-v1.json` 供应。selected LFE gate与PRIMARY/LFE mantissa边界见F09LB；其他profile仍不支持，变换和整链覆盖继续见Q05。

## F09.1 配置预算

当前六块 profile 的预算系数为 `F32(0x3eaaaaab)`。先计算 `q=rne_int(F32(0x3eaaaaab)×2^31)=715827904`，再用整数求 `B=(q×6×1024)>>31=2048` 个 16 位 words；帧预算为 `16B=32768` bits。

当前保留预算为 0，available=B。普通刷新使用 mode 1、secondary mode 0；反向策略 mode 0。POWER bias=0，指数 optimizer option=1，主全带宽 error gate 为 `int(available<896)=0`。LFE 的独立 error gate 在selected profile中为0；其冷初始化、条件writer来源和独立于FBW gate的接口由F09LB定义。

## F09.2 两套指数

主路系数为 `a[k]`，伴随路系数为 `b[k]`，均为已经具有 F32 精度的输入。POWER 值为 `min(1,(a[k]⊗a[k])⊕(b[k]⊗b[k]))`。全带宽使用前 217 bins，LFE 使用 7 bins；POWER 只用于刷新和策略选择。

对 F32 位模式 `w`，定义无符号 `d=(2w) mod 2^32`，`e=(d>>24)-126`（d=0 时 e=-127），`h=sat[0,60](-e)`。POWER 要求有限非负值，允许两种零符号。其 raw 指数为 `min(48,W16(bias+h))>>1`；在当前 bias=0 下范围为 0～24。不能用 log2、取绝对值后估算或 PRIMARY 算法代替。

PRIMARY 只消费主路系数。先按严格 `<-1` 和 `>1` 比较把超范围值夹到 -1 或 +1；范围内位模式不变，包括 signed zero 和 subnormal。由 clipped 位模式求上述 h，再取 `min(24,h)`；不加 POWER bias，不除以 2。全带宽 clipped 系数必须用于后续 mantissa 量化，不能仅改指数。LFE MAIN与FBW PRIMARY调用同一strict clip原语；超范围系数写成exact F32 ±1，等号及范围内位模式保留，mantissa必须消费clipped staged row。完整边界和错误/提交语义见F09LB。

PRIMARY 的写入不会改写较早保存的 POWER 快照。converter 使用该 POWER 快照重新选择策略。

## F09.3 刷新

每通道每帧工作指数行冷置为 0；block 0 必须结构刷新。全带宽的误差比较端点为前 128 bins，行存储为 217 bins；LFE 为全部 7 bins。刷新只修改指定范围。

有结构刷新时，工作行第一项取当前行，后续项按递增频率把每项上界限制为前一工作项+2；比较为 `>=`，写回按 W16。该路径不计算误差门限。

三个刷新分支的 begin/end 均严格采用 `[0,128)`（全带宽）或 `[0,7)`（LFE）。结构刷新不修改 current 行，只在此区间建立/限制 work；普通 mode1 在此区间限制 current，接受时复制同一区间到 work，拒绝时保留该次 current 修改及旧 work；LFE mode0 在同一区间边比较边更新 work，current 保持输入值。全带宽索引 128～216 在这些刷新步骤保持输入位值，不能延伸限制或复制到尾部。

普通 mode 1 的复用候选会先在当前行施加逐项向上差值不大于 2 的限制。误差索引为 `trunc(W32(abs(current[i]-work[i])×weight[min(i,3)])/10)`。误差映射按F09.6生成；4个位置权重和普通threshold从 `parameters/policy-authorized-v1.json` 按字段身份读取。文件/必要字段缺失报 `PARAMETER_UNAVAILABLE`；profile ID或配置字段不符报 `UNSUPPORTED_PROFILE`；字段shape、类型、位模式或hash不符报 `POLICY_PARAMETER_ERROR`。

误差累加采用 W16；左比较值为 `R32(sum)⊗2^-15`，右比较值为 `(R32(count)⊗2^-15)⊗threshold`。仅左值严格大于右值时刷新，并把当前行复制到工作行。拒绝刷新时已经修改的当前行保留，不能回滚它。

LFE mode 0 的候选在比较期间更新工作行。首项直接取当前值；后续项先限制到前一已更新工作项+2，再与该项旧工作值计算误差表项。该模式的向上限制条件为 `>`。阈值比较同上，标志只有 1/0。

## F09.4 主反向策略

六块从 5 到 0 扫描；距离初始 0，每访问一块 W16 加 1。刷新块根据距离选策略：距离小于 2 取 3；2～3 取 2；至少 4 取 1，随后距离归零。非刷新块取 0。

short flag 在此选择之后覆盖策略为 3。该覆盖不重新计算距离。策略语义：0 reuse、1 span=1、2 span=2、3 span=4。

LFE 直接使用刷新标志作策略，不调用此反向规则。

## F09.5 格式匹配

五个通道的六块策略与公开 32 项 frame pattern 表按索引递增顺序匹配，首次匹配决定 frame code。表的公开来源为 ETSI V1.4.1 Table E.1.9，允许从对应公开文档取得。若任一主通道没有匹配，使用逐块策略格式；本 profile 无 coupling。主与 converter 分别匹配，不能共享主 frame code 结果。

converter 无匹配时返回失败，保持原输出；不能强行选接近的表项。生产内置的歌曲特定策略序列不是允许的依赖。

## F09.6 完整刷新误差映射

D06-E 是研究观察后归纳的数学行为规则，不声称原创设计来源。输入 `index` 必须为严格整数 0～31；bool 不算整数。输出 I16 按精确整数运算生成：index<2 时为 0，index=2 时为 1，其余为 `2×index-4`。不做浮点运算，不用逐项拟合参数。

生成资产为 `parameters/exponent-refresh-difference-math-v1.json`，包括上述规则和由规则生成的 32 项功能值。其 64 字节 I16LE SHA256 为 `a0e75dafa9f3c1627ec36351ed857711c5a79f60fd7fda0b9f988f0505a4a070`。规则在完整声明区间的 32 项均与研究观察一致；原表文件未搬入隔离包。

普通 mode 1 仍先完成 F09.3 的 current 修改、W32 乘法和向零除法，再用所得 index 查此映射。LFE mode 0 先计算经限制候选与该项旧 work 的绝对差，再查映射。映射规则不替代这两套 index 算法，也不解除 threshold 或位置权重依赖。

固定 raw/current/work 指数的范围为 0～24。生产权重必须为 4 个 I32，满足本已验证 profile 的 `[0,10]` 界限。索引超出 0～31 时返回 `EXPONENT_INPUT_ERROR`，按 I03 停止并重建；不夹索引，也不扩展查表。配方向量见 `test-vectors/EXPONENT-REFRESH-DIFFERENCE-MATH-VERIFICATION.json`，涵盖 32 个有效索引及 7 个拒绝输入。

## F09.7 帧计划所有权与试算连接

同一个 source frame 提供 `installed_six` 和 `delayed_six` 两组 OWN 音频，每组为 6×1536 个有限 F32。两组属于同一帧号，但时域不同。瞬态和结构 flag 消费 installed；gain、db、PRIMARY/companion 变换及 LFE 变换消费 delayed。外部顺序为 L、R、C、LFE、Ls、Rs，内部映射为 `[0,2,1,4,5,3]`。

帧内按块 0～5 推进。全带宽 work 为每帧每通道新建的 217 项 I16 零行，LFE 为每帧新建的 7 项零行；仅在该帧六次刷新调用间延续。它们不是上一帧 work。跨帧状态另包括五路 PRIMARY/companion 各 128 样本历史、LFE A/B 各 128 样本历史、gain peak、db selector、瞬态状态和附加 flag FIFO；完整冷状态按各原语定义初始化。

音频计划只计算一次，返回完整帧计划与下一份 OWN numeric state。主编码 SNR feasibility 重用这份固定计划，重新计算候选 BAP、mantissa 和容量；不能重新推进 mean、db/gain、指数刷新或 normalization。选定搜索先试 candidate 16，再在 `[16,784)` 进行最多 10 次 midpoint 试算；初始候选不计入这 10 次。成功更新 lower 和 best，失败更新 upper，最终提交 best。

主编码变换写 staged copy，只在找到 best 后一次性提交。已有研究流程在独立 producer 中先连续生成 numeric plan，早于下游编码执行；下游失败不提交其 staged state，不等于自动回滚已生成的上游状态。独立实现失败后按 I03 停止、整体重建；若支持重试 source frame，必须从明确保存的完整帧前 OWN 状态恢复，不能只恢复 selector。

reset 恢复所有原语的冷状态并从 source-frame 0 开始。EOF 不新增 db 块、零指数行或 selector flush；只处理调用方按普通音频时钟实际交付的完整帧。发布 AU 数不能代替音频计划的 source-frame 数。

缺少必要参数返回 `PARAMETER_UNAVAILABLE`；输入 shape、类型、非有限 PCM 或域外配置返回 `EXPONENT_INPUT_ERROR`；F32 运算出现非有限中间结果返回 `NUMERIC_FAILURE`。整数不接受 bool。失败不发布正常帧计划，按 I03 停止和重建。参数集变更也要求重建，不能在同一流水线中途替换。

本轮研究核对的是实际全歌曲 policy Python 计划：4249 个连续 source frame、25494 个 db/gain 完成块，9 个 Python source 身份与该运行记录一致。该执行不同于较早 minimal composition 的 metrics-only codec 运行。本轮没有重新执行真实歌曲或 SDK；800 个 refresh、8032 个 selector、100 个均方合成比较在研究侧完成，未把未授权参数或这些参数的输出 trace 搬入实现包。局部证据不代表整个音频包已可实现。

D06-E数学映射与参数化policy文档r2局部独立审查通过：32有效、7拒绝、完整64字节hash，LFE策略32有效及18拒绝均通过；LFE输入域歧义已关闭。19个必要标量现由F09PA授权provider供应。完整normalization全输入域独立证明和实际整曲policy运行仍未完成，不表示整个policy/codec生产链通过。
