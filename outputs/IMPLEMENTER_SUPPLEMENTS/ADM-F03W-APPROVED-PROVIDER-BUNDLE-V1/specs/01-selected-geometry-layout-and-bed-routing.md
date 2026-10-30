# 固定前端几何、布局展开与床路由规则：内部授权参数版本

功能标识 `F01GL`。仅供内部隔离实现。这里交付可执行规则与合成向量，不交付原表文件、研究源码或完整35项原几何。生产所需的唯一特殊标量已获用户明确授权内部交付。`parameters/geometry-front-side-y-v1.json` 供应一个 I32 Q15 功能值；不是原35项表，也不声称该值来自独立数学设计。

## 1. Layout bit 的连续占位规则

输入 layout 是 U32，但数值必须小于 2^19；拒绝 bool、负数及更高 bit。profile_enabled 必须为 bool。19 个 layout bit 依次占用静态输出 bit 的宽度为 `(2,1,1,2,2,1)`，随后 13 项宽度均为 2。每项从上一项的末尾继续，不留洞。

初始 offset=0。第 i 项的宽度为 w_i，expansion_i=`((1<<w_i)-1)<<offset`，然后 offset+=w_i。最终 offset=35。对 layout 已设 bit 的 expansion 求 OR。profile_enabled=true 时再 OR bit35。bit35 是配置标记，不是第 36 个点。

枚举 bit0～34 的递增索引形成稳定 basis 顺序。不按空间坐标重新排序输出；后续 panner 的分组另有稳定排序规则。19 项生成值及 524,288 个无 profile 输入逐项等价。profile 变体只对双方结果 OR 同一个独立 bit35。

这项规则恢复了连续单/双通道布局的功能关系；不声称它来自公开标准或独立发现了原厂设计历史。其交付依据是用户允许内部整理几何、布局和映射行为。

## 2. 参数化完整 geometry 原语

通用原语输入 `static_geometry[35][4]`。每行前三项为 signed I32 Q15 XYZ，第四项为 U32 fallback，范围 0～35，35 表示无 fallback。严格检查整表形状、类型与范围后才生成局部输出。参数在 programme 初始化时冻结，reset 时重新安装；没有跨 callback 的累积状态。

每个启用索引 i：如果 fallback!=35 且 fallback 对应 mask bit 未设，则使用 fallback 行的 XYZ；否则使用本行 XYZ。只跳一次，不递归，也不使用 fallback 行自己的 fallback。若 profile bit35 设且有效 Z>0，把有效 X 按严格 `X>16384` 转为 32767，否则为 0。Y/Z 保持。

各坐标先 R32(I32)，再 R32(所得 F32×2^-15)。不能用另一个饱和器处理参数。这个原语本身不供应完整原表。没有值供应时不能默认补零、默认网格或假造未消费记录。

## 3. Selected P11/P15 的直接生成器

生产供应器与上面的完整原语分开。选定配置只支持 layout=0x111B 或 0x159B 且 profile_enabled=true。其他配置报 `SELECTED_GEOMETRY_PROFILE`，不伪称已供应全部 35 点。

定义 U=2^15−1、H=2^14。先按以下规则生成下层点：前左 `(0,0,0)`、前右 `(U,0,0)`、前中 `(H,0,0)`、中侧左 `(0,H,0)`、中侧右 `(U,H,0)`、后左 `(0,U,0)`、后右 `(U,U,0)`。

P15 在这七点后追加前侧左 `(0,Y_front,0)` 和前侧右 `(U,Y_front,0)`。`Y_front:I32 Q15` 是必须取得的特定非秘密功能参数，要求 0<Y_front<H。当前授权参数供应 Y_front=5285。按参数文件的 I32 类型读取，转换后 F32 word=0x3E252800。若缺少该参数文件，本供应器仍报 `PARAMETER_UNAVAILABLE`。P11 不读取它，不因为 P15 缺值阻止 P11 的数学生成。

随后追加上层前左 `(0,0,U)`、前右 `(U,0,U)`。只有 P15 再追加上层中侧左 `(0,H,U)`、右 `(U,H,U)`。最后两种 layout 均追加上层后左 `(0,U,U)`、右 `(U,U,U)`。坐标按第 2 节的两次 R32 转换。

P11 总数 11、P15 总数 15。P11 来源索引为 `[0,1,2,4,5,6,7,13,14,21,22]`。P15 来源索引在其下层七点之后加入 11/12，上层前点之后加入 17/18。这些索引是布局身份，不是额外交付原表。

在两种 selected mask 中，fallback4→6、5→7 的目的 bit 均已设，实际没有跳转。原上层 X 的非网格数只影响左右判定，profile 规则将其覆盖为 0/U。直接生成器消除了这两个不再必要的原值，不声称 profile=false 的 X 也等价。

研究侧已逐词比较 selected P11 的 33 个 F32、P15 的 45 个 F32。P15 比较使用已经获准内部交付的 Y_front。完整 P15 的45个 F32 words、F32LE180bytes和两种 selected basis 的全部78词见 `test-vectors/SELECTED-GEOMETRY-AUTHORIZED-VERIFICATION.json`。授权不替代算法核验。P11 132-byte F32LE SHA256=`ea7577cf5fa0bce6ab76aeb81447c452cbac498fa79bd039a11149d970d7de68`。

## 4. Bed label 的功能路由

F01G 已 trim 单个 label。这里只接受字符串，取从第 4 个字符开始的后缀。长度<=3 得到空字符串。不是仅允许 RC_ 前缀，也不全局删除前缀；不做大小写折叠或第二次 trim。

语义路由为：L→前左；R→前右；C→前中；LFE→唯一 direct LFE；Ls/Lss→中侧左；Rs/Rss→中侧右；Lrs→后左；Rrs→后右；Ltm/Lts→上层中侧左；Rtm/Rts→上层中侧右。内部 bed_id 按以上十个角色依次为 0～9，其中 LFE=3。

其他后缀拒绝，包括 object、Lfh/Rfh、Lrh/Rrh、Lw/Rw、LFE2。不得为这些名称找最近位置。算法没有必要加载或供应原三张 native name/enum/code map 文件；这份语义规格只给出它们组合后的实际床路由行为。若另一个模块确实消费 native enum 或中间 code，应另列接口，不将内部 bed_id 充当它们。

输入一个 DirectSpeakers master/pack 的引用图。按 pack 的直接 channel refs 文档顺序建立 bed_input_id=master semantic key+j，检查 U32 溢出。每个 channel 正好一个 block/label；pack 引用互异且与 CHNA 已解析 bed channel 集合完全相同。保留 channel_format_id、host_channel_index、bed_input_id 和 bed_id，四者不能互换。

## 5. Bed normalized source 点

bed_id=3 不生成 regular source 点。其余九点由矩形语义生成：前左/右的 y=z=0，x=0/1；前中 x=1/2,y=z=0；中侧左/右 y=1/2,z=0；后左/右 y=1,z=0；上层中侧左/右 y=1/2,z=1。左右 x 均为 0/1。

这些坐标直接是 exact F32 0、1/2、1，不先转成正 Q15 饱和端点。它们不同于第 3 节 prefix 的 U/32768。完整九点 27 个 F32 与原模型逐位等价；其生成规则源自 normalized rectangular role，source 身份的对应关系属于 recovered 功能语义。

prefix/bed source attribute 的字段为 `(0,0,0,3,2)`。字段最大值依次是 1、7、15、3、3，packed=`f0|(f1<<1)|(f2<<4)|(f3<<8)|(f4<<10)`，固定值为 0xB00。对象属性来自本 callback sampler，不能套用该常量。

## 6. 固定 48-source 安装与身份关联

selected regular mode=1、constructor_mode=0、prefix_count=11。按物理 input control 行 0～37 递增检查，每行号互异且完整。branch_flag=1 表示 bed，=0 表示 object。只有 bed 且 code_offset20=3 的行跳过；正好一条 direct LFE。其余 37 行按物理次序映射到 source11～47。source0～10 属于 prefix。LFE 物理索引不硬编码。

prefix 几何先生成。每个床点先绑定自己的 `channel_format_id` 和 semantic role，再由 F01E.6 查到实际物理 control 行。对象按自己的物理 control 行安装。跳过唯一 LFE 后，37 个 regular 行按物理行递增顺序紧密映射到 source11～47。必须核验 source0～47 各有一份 XYZ/attribute，无重叠、无缺失。

当前真实输入恰好是 bed slots0～9、object slots10～37。实现不能依赖这个顺序。pack 引用与物理 bed 轨道按完全相等的 `channel_format_id` 逐条连接；10 个 bed slots 与 28 个 object slots 必须无交集且完整覆盖 0～37。合成验收包含 pack 重排和 bed/object 交织，再按物理行 compact；不能只检查集合相等后按数组位置配对。

源模型只完整定义 panner 会读取的 source 字段。prefix/bed 的 +4 skip=0，+8=0、+12=3、+16=2、+20=0、+24 XYZ、+36/40/44/48=0、+52=1。对象则由本 callback 52-byte control 覆盖 offsets4～55。source+0 的 hold/state 字段不属于本模型恢复结果，不能伪造成 source ID 或组装全 56-byte 原结构后声称全字段已闭合。

F03P 的 feature_width=正零只影响 feature-source 视图。保留原音频 width；不能据此把 F04G spread 分支关掉。对象的 XYZ、flags、width、attribute 按 sampler/事件的本 callback 更新，与几何静态供应器分别拥有。

## 7. 错误与验收界限

输入 type/shape/range 错误先失败；layout/profile 域、参数取得状态随后检查。纯几何供应器不提交运行时状态。组合 pipeline 遵循 I03 fail-stop；不要借局部原模型 ValueError 声称恢复了所有 native 异常提交过程。

局部向量提供 6 个合成 geometry、38 个不同 LFE 物理位置的 route、53 个 label、12 个拒绝检查。没有在 clean 合成 geometry 中加入生产特殊坐标。19 mask生成和全部 524,288 layout 等价由研究侧报告锚定。selected P11/P15 的生产词比较没有重新渲染完整 ADM/音频，也没有最终空间效果验收。

Y_front、selected basis、固定 profile 的 ADM/DBMD/单事件 sampler 和 `channel_format_id` pack/physical 连接均已供应。r3 向量显式包含 interleaved join、修正后的 sentinel、364 个真实 control 词和失败状态；仍待无研究上下文复审。本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
