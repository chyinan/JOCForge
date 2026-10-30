# F01B Bed 路由与 source geometry 安装

本条款连接 F01G 的引用图与 F03R 的 source 安装。bed/prefix producer 以 `channel_format_id` 连接 pack 与物理轨道。对象事件的默认、DBMD 合并、sampler 和 control52 见 F01E。

## F01B.1 Bed 引用与身份

接受分支要求一个 bed master object、一个 DirectSpeakers pack。pack 的直接 audioChannelFormatIDRef 按 XML 文档顺序读取，trim text；引用必须互异，并与 CHNA 已解析 bed channel 的集合完全一致。每条 bed 轨恰好关联一个 master object；每个 bed channel 恰好一个 block、一个 speakerLabel。

按 F01G.5 的 AO_XXXX 规则取得 master semantic key。bed input ID=该 key+pack 引用序号，要求 U32 无溢出。这个 ID 与 host_channel_index 不同；保留二者，不能用轨号代替 ID。非零 master start 不属于选定域；DBMD 和事件安装见 F01E。

speakerLabel已在F01G层trim。所选profile的规范路由由F01GL.4直接执行suffix→semantic role→bed_id；本文件旧的native三层映射描述仅保留为历史诊断接口，不是实现依赖。路由规则取该字符串从第4个字符开始的后缀；长度<=3时后缀为空。不是全局删除某个前缀，也不另做大小写折叠。后缀须在F01GL明确语义集合中精确匹配，否则拒绝。

独立实现直接产生semantic role和bed_id，不需要取得native enum或channel code，也不加载D10-BR原映射表。不能把未支持的height/wide名称自动映射到最近扬声器。输出按pack引用次序保留channel ID、host slot、input ID、parsed suffix、semantic role和bed ID；若诊断层另需native enum/code，必须作为非产品诊断接口单独提供，不能反向改变语义路由。

direct LFE的语义bed ID=3，F03R仅从regular路径跳过它。其他bed ID的矩形normalized点由F01GL.5生成，不能把缺失点补零。Ls/Lss、Rs/Rss、Ltm/Lts、Rtm/Rts别名已在F01GL明确；native enum/code原值不属于所选实现输入。

## F01B.2 Layout 与 Q15 几何参数

selected constructor_mode=0、regular routing mode=1；prefix layout=0x111B，profile_enabled=true。这些配置量与 selector mode、codec acmod 不同。

参数 layout_expansion[19] 为 19 项 U64LE mask（152 bytes，D10-LX）；static_geometry[35] 为 35 个16-byte记录，各按 X/Y/Z 三个 I32LE Q15、fallback U32LE 顺序存储（560 bytes，D10-G）。每个 fallback 范围 0～35，35 为 sentinel；不能读取不存在的第 36 个记录。

layout 必须在 [0,2^19)。对每个已设的 layout bit，OR 对应 expansion 项；profile_enabled 时再 OR bit35。仅枚举 mask 的 bit0～34，按 index 递增，构成 prefix output 顺序。选定安装要求恰好 11 项，否则报 LAYOUT_DOMAIN，不裁剪或补齐。

对每个启用记录，若 fallback!=35 且 mask 中没有 fallback bit，则只读取该 fallback 记录的 XYZ；不递归追踪其 fallback。否则使用本记录。profile bit35 已设且有效 Z>0 时，X>16384 将 X 改为 32767，其余将 X 改为 0。比较在 I32 Q15 域完成；Y/Z 保持。

XYZ 逐项先转 F32，再乘 exact F32 2^-15。没有隐含 clamp；静态参数越界、finite 或存储问题应在 provider 安装时拒绝。输出 11×3 F32 和稳定 prefix 索引。

source attribute 五字段范围依次为 [0,1]、[0,7]、[0,15]、[0,3]、[0,3]。U32 pack=f0 OR (f1<<1) OR (f2<<4) OR (f3<<8) OR (f4<<10)。prefix 固定字段为 (0,0,0,3,2)，因此attribute=0x00000b00；F05H消费该packed值。

## F01B.3 Bed 点与对象点

bed_points 为 9 个 {bed_id:I32,XYZ:3×F32} 记录（D10-BP），bed_id 互异，XYZ finite。先以 bed_id 绑定 semantic role，再以 `channel_format_id` 连接到物理 control 行；跳过 direct LFE 后按物理行 compact 到 prefix 后。每个 bed 点同样使用 attribute 字段 (0,0,0,3,2)。这组点是路由点，与静态 prefix basis 不是同一参数表。

对象点从 sampler 的当前 XYZ、source attribute 取得，按 F03R 的 control→source 映射安装。必须验证每个 source 0～47 恰有一份 position 和 attribute，且 object 与已安装 bed/prefix 不重叠。完整 sampler 及其构造权重仍由 Q03 跟踪。

pack ordinal 只确定 bed input ID。床点、音频和 control tail 都在身份连接后按物理轨道次序安装。F01E 的合成向量覆盖 pack 重排及 bed/object 交织；10 个 bed 与 28 个 object slots 必须完整覆盖物理域 0～37。

## F01B.4 依赖、生命周期与拒绝

所选P11/P15、layout expansion、bed route和normalized bed点由F01GL完整定义。P15唯一特殊Y由 `parameters/geometry-front-side-y-v1.json` 供应；缺该文件报PARAMETER_UNAVAILABLE。D10-BR/BP/LX/G原表不再是所选profile运行依赖，也不读取研究metadata缓存。profile=false或其他layout保持不支持。

layout/static 参数在 programme 初始化时安装并冻结；新 programme/reset 重新安装。对象坐标的跨 callback 更新必须由 sampler/event 规则产生，不能重复用首次值。bed/prefix 的静态安装也不替代 F03R 每 callback 的 flags。

引用不完整、label/enum/code 不支持、重复参数 key、fallback 越界、source 冲突或 geometry 缺失都拒绝，遵循 I03。F01E 另提供完整身份冲突和 programme fail-stop preimage。见 `../test-vectors/BED-GEOMETRY-LOCAL-VERIFICATION.json`：合成 expansion/static 参数只验证几何算法；F01E 的完整向量验证物理身份连接。
