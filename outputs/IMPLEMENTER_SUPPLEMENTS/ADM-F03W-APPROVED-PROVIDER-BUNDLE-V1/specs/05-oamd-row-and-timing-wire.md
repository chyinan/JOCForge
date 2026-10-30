# F05O 当前 OAMD 单帧、对象行和 section 封装

本条款定义显式模型对象行到 Type11 body 的单帧路线。当前编码 programme 为可选首行 LFE 加对象行，目标为 LFE+15 个对象。非 shortcut bed、多个 metadata frame、非零 width、position extension、ancillary 或非 nominal/zero gain 分支尚不支持，不能默默清零后编码。

## F05O.1 行接口与坐标量化

行顺序不可更换。仅首行可为 LFE，其余必须为 object。每行有 mute 整数0/1、gain 整数、spread 整数、object 的 xyz_q15[3]、component U3、z_valid/diffuse 整数0/1、width_q15 和额外 ancillary 值。当前 width 和 ancillary 必须为0，spread 在 unmuted 时必须为32767。

object 输入域 x/y∈[0,32767]、z∈[-32767,32767]。令 d=v-16384，XYcode(v)=31+sign(d)×floor((62×abs(d)+16384)/32768)，sign(0)=+1。Zcode(z)=sign(z)×floor((15×abs(z)+16384)/32768)，sign(0)=+1。运算为整数，XY 范围0..62，Z 范围-15..15；不以浮点 round 替代该公式。

坐标来自空间控制时，先按 N00 取 F32，再计算 trunc(R32(coord×32767))。F05H的52-byte投影首词inactive_pair非零时令三个整数坐标为0；该转换本身仍生成 unmuted、gain4096、spread32767 的对象行，不能把inactive_pair直接当作 wire mute。component 来自对应空间槽，当前控制 producer 只生成0..5；z_valid=(控制值非零)。F05L 定义15行shape、同callback代次、LFE拼接和失败提交；附加几何分支继续拒绝。

## F05O.2 全局 header

MSB-first 写 version:2=0。row_count 支持1..159：count<32 写 count-1:5；count>=32 写31:5，再写count-32:7。写 shortcut:1=1、leading_lfe:1、protected_sections:1=0。

section_count:4 为1（只含 rowdata）或2（另含 trim）。不写任意 section 或未知扩展；trim 有完整独立 body 规则时才可加入。当前 auto trim 的数据绑定和 trim body 仍是必要后续条款。

## F05O.3 Section 长度的 escaped 整数

每个 section 为 kind:4、escaped(payload_bytes-1)、payload。rowdata kind=1，trim kind=2。payload 先写 control:1=0，再接该 section 的内容，末尾补零到整 byte。section 的 payload 字节边界相对 payload 自己计算，不要求外层当前 bit 位置已对齐。

escaped 为最多四组的 bijective base16 表示，每组写 digit:4，紧跟 continue:1。对非负整数 n，其唯一形式满足：首 digit 为d0，后续每步 n_next=(n_previous+1)×16+d；最后一组 continue=0，前面均1。等价构造：末 digit=n mod16；若 floor(n/16)=0 则停止，否则对 floor(n/16)-1 继续分解；最后逆序发送各 digit。拒绝超过四组、负值、未终止或截断。

最后整个 body 再补零到 byte。读者验证 section 内及最终尾部最多七个零 padding bits，不允许额外未解释字段。

## F05O.4 单帧 timing

base=0 写 base_mode:2=0；base 为8/16/18/24时写mode1，再写这四值的ordinal:2；其他合法 base U5 写mode2再写base:5。mode3保留，拒绝。写 frame_count_minus_one:3=0，再写 position:6。

duration 为0/512/1536时写 duration_mode:2=0/1/2。其他 duration 写mode3，若在下表则写 table_flag:1=1和ordinal:4，否则写flag0和duration:11。表按ordinal0..15为 [32,64,128,256,320,480,1000,1001,1024,1600,1601,1602,1920,2000,2002,2048]。duration=2048必须使用表分支，不能写入 U11。

precision=32 写 default_precision:1=1；其他 precision U5 写flag0和precision:5。当前通常 base0、position0、duration1536、precision32；这些是本接口的明确值，不是从参考 metadata 推断的歌曲常量。

## F05O.5 每行 body

按行顺序，先写 mute:1。mute=1 时跳过 gain、spread、object 空间字段，但仍写末尾 trailing_extension:1=0。

unmuted gain 的选择有严格优先级：若不是首行，上一行也 unmuted 且其原输入 gain 与当前相同，写gain_mode:2=3；否则gain在[3866,4337]写mode0；否则gain=0写mode1；其他gain拒绝。mode0解码为4096，不保留原区间内的细微差异；mode1解码为0。当前不写mode2。

随后写 full_spread:1=1。LFE 不写 object 字段；object 依次写 XYcode(x):6、XYcode(y):6、Zcode>=0:1、abs(Zcode):4、position_extension:1=0、component:3、z_valid:1、width_mode:2=0、ancillary_presence:1=0、diffuse:1。最后写 trailing_extension:1=0。component 的编码域为0..7，但当前模型控制 producer 只供0..5；不因字宽扩大 programme 支持范围。

## F05O.6 解码与反馈依赖

解码 rowdata 必须使用同一 row_count 与 leading LFE 生成的 kind 列表。gain mode0/1为4096/0；mode3取上一 unmuted 行已解码 gain，首行时按4096，非首行且上一 muted 时按0。这个解码规则不说明编码器会产生这两种非常规 mode3 情况。

mute 行 decoded gain/spread=0；默认 object 坐标为(16384,16384,0)、component0、z_valid1、diffuse0。unmuted object 的 XY U6先 clamp到62，再索引 XY decode table[63]；Z magnitude U4索引 Z decode table[16]，按符号位取正负。component code0..6保持，7解码为0。所有未支持 extension、spread、width 或 ancillary 分支拒绝。

XY/Z decode grid现由PG06提供完整整数生成规则和数据：[oamd-coordinate-grid-q15.json](../parameters/oamd-coordinate-grid-q15.json)。XY[j]=min(32767,floor((32768j+31)/62))，j0..62；Z[j]=min(32767,floor((32768j+7)/15))，j0..15。79项与研究值逐位一致；不需要取得原D11-XY/D11-Z编译表。上述编码公式和decoder回写网格是两个步骤，不能把原输入Q15直接当作解码值。完整反馈消费者的状态仍须按自己的类型绑定。

## F05O.7 状态和验收边界

本 writer 单次消费显式 timing/rows/trim，不保存上一 callback 的对象行。gain reuse 只作用当前 body 的相邻行。F05H拥有跨callback metadata history，F05L显式提供timing、15行shape、leading LFE、trim、writer/source时钟和ID11 queue；不能用本 wire 原语替代这些live状态。

接收者拒绝重复 section kind、缺 rowdata、未知 section、非零 section control、截断、未解释尾部或非零 padding。当前 writer 只输出1/2个 section，且只输出一个 metadata frame。

wire原语验收包括count31/32/159、escaped16边界及最大组数、默认/显式/表中timing、XYZ端点和中心、相邻gain reuse与muted前行、LFE/object不同字段、section自己的对齐及最终对齐。选定profile的trim、DBMD、metadata history、空间控制→15行→完整body→160-byte queue生产关系见F05L；F05L r5 fresh独立局部审查已通过；真实ADM整链仍未运行。
