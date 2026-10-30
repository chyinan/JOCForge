# F05RT marker/history、Type11、reader56 与 target updates 组成

状态：`CLEAN_DRAFT / PASS_MODEL_POSTCOMPARE / FRESH_LOCAL_REVIEW_PASS`。本条连接 F05H、F05O、F05T 与 main receiver 的 target producer。它定义 15 行 regular marker/history、Type11 单帧回读、16 行 reader56 和 `target_updates[16][6]`。fit-control record56 与 reader record56 是不同类型，不能按相同长度互换。

r3 独立 package-only 复审从输入重算 96 条主 reader record、16 条 mute edge record、576 个 panner F32 words、4 个 panner 拒绝、7 个事务拒绝和 7 次 reset replay，差异为 0；r2 的 mute edge HIGH 已关闭，OAMD reader-target binding 仍与当前 schema 和 provider bytes 一致。精确冻结与结果哈希见 `../test-vectors/READER56-TARGET-LOCAL-REVIEW-STATUS.json`。该结论使 F05R56 选定局部范围 ready=YES；真实全曲 producer、同 runner F04C 和 cold ADM→wire/player 空间验收仍使整链 ready=NO。

## F05RT.1 输入、状态和时钟

frontend callback 使用零起始 `g`。每次输入包含：

- current target XYZ、ID、attribute 和 assignment active，各15行；
- active target 索引和对应 `n×48` matrix；
- 48个 source flags；
- 15行 callback 开始时的 previous overlap 和当次 mixer；
- `reader_enabled`。

状态保存 callback_index、retired、三槽 metadata history、previous active、上一代 current target snapshot、16个 reader56 和 reader invocation 计数。history 每个 frontend callback 都推进。reader 在两次 direct-LFE priming 后启用：`reader_enabled=(g>=2)`，reader invocation 为 `q=g−2`。

history payload 保留自己的 producer generation。cold record 使用语义 generation `−1`。新 record 使用当前 `g`。Type11 body 和 reader invocation 属于当前 envelope `g`，但各 regular target 的内容 provenance 取 delivered history row 的 generation。字节相等不能替代 generation。

## F05RT.2 fit record 与 marker

按 F05H.1 从 current XYZ/attribute 构造15个 fit-control record56。marker 的 previous XYZ、ID 和 attribute 取状态中的上一代 current snapshot。previous active 取状态中的 marker after。禁止调用者另传一份可冲突的 previous snapshot。

quiet witness 按 active target ordinal 和48列 source顺序查找。source flag必须为0，matrix coefficient必须严格大于0。没有witness时quiet=1，并把current active置0。

changed 为 ID变化、attribute变化或XYZ距离平方严格大于 `R32(0.1)`。marker优先级保持 F05H.3：新活动为−1；否则changed时比较old/new前1536样本能量的严格100倍关系，选择1、−1或0；其余为0。record word0写quiet，word1写两代都inactive。energy使用 previous overlap 和 mixer head，不消费 mixer tail。

## F05RT.3 三槽 history 和52-byte投影

每行先执行 `S0←old S1`。随后 marker −1 写 S0/S1/S2，0写S1/S2，1只写S2，其他I32不插入。更新完成后，如果S0.word1和S1.word1都非零，则完整执行 `S1←S0`，包括payload、generation和producer row。

当前 callback 的 delivered record 是更新后的S0。丢弃word0后得到52-byte control。15行顺序保持 target slot0..14。history after、previous active after和current target snapshot只在完整 callback 成功后提交。

## F05RT.4 Type11 单帧与 selected profile

reader启用时，把15×52 control交给 F05O.1 adapter。adapter在最前面插入一个 coded direct LFE，形成16行 Type11：row0为kind0，row1..15为kind2 object。regular投影必须满足当前 zero-width、无ancillary、component0..5分支；不支持字段返回 `R56T_PROFILE_UNSUPPORTED`。

Type11 timing固定 base0、position0、duration1536、precision32。trim来自已核验 DBMD 默认分支：mode3、9个group flags `[1,0,0,1,0,1,1,0,1]`、九组默认 `[16,16,16,0,0]`、16个row flags全0。完整值和provider身份见参数文件。

writer按 F05O 编码。reader必须从wire bits重新解码，不能复用writer输入rows。XY/Z decode grid使用 `oamd-coordinate-grid-q15.json` 的63/16项生成数据。decode失败不产生reader或target输出。

## F05RT.5 reader56 更新

cold有16个reader56。每个record的14个U32初始为0，只有word11=3。每次reader invocation都提供上一invocation的完整16×56字节；不能每帧重新cold，也不能用fit-control history当previous reader state。

whole-state SHA-256 使用 `READER56-TARGET-STATE-SCHEMA.json` 的 `encode_node-v1`。dataclass 节点逐字节写 `D || U16LE(class_name字节数) || ASCII class_name || U16LE(field_count)`，随后严格按声明字段顺序，对每项写 `U16LE(field_name字节数) || ASCII field_name || encode_node(value)`。tuple/list写 `T || U64LE(count)` 后紧接各嵌套节点；bytes写 `B || U64LE(length) || payload`；I64、bool和none分别使用schema指定的 `I`、`Z`、`N` tag。任何层级均无NUL、padding、分隔符或尾随字节。schema同时交付9047-byte cold完整hex preimage及其SHA-256，消费者必须先由该preimage复算cold digest，再接受连续向量中的whole-state SHA。

row0写 direct record：word0=1、word1=4。object row写word0=2、解码后的XYZ、component映射、diffuse、z_valid和extra。mute object 必须使用 wire decoder 的 canonical decoded row：XYZ=`[16384,16384,0]`、component=0、diffuse=0、z_valid=1、extra=0、gain=0；不得把 `SourceRow` writer-side 构造默认值 `[0,0,0]` 和 gain 4096 当作 decoded-row 输入。随后所有行更新word9=1、word10=1、word11=source-state、word12=gain、word13=row flag。source-state读取previous word11；unmuted时保存 `1+int(1<=old<=2)`，所以cold3→1，后续1→2并保持2。当前row flags全0。

reader56完整更新后才计算targets。reader state和target输出来自同一份新reader records。

### R56_WORD_SCHEMA_V2

reader56 是 14 个 U32LE 词，共 56 bytes。word3 和 word7 可承载 I32 的二补码位模式。所有未写字段都按下表处理，不能按结构体 padding 推断。

| word | direct kind0 | object kind2 | mute object |
|---:|---|---|---|
| 0 | U32 1 | U32 2 | U32 2 |
| 1 | selector U32 4 | X Q15 U32，0..32767 | U32 16384 |
| 2 | U32 0 | Y Q15 U32，0..32767 | U32 16384 |
| 3 | U32 0 | `z_valid!=0` 时写 Z I32 二补码，否则写 0 | U32 0 |
| 4 | U32 0 | component code 映射 `[0,3,2,4,1,5,6][component]` | U32 0 |
| 5 | U32 0 | diffuse U32 0/1 | U32 0 |
| 6 | U32 0 | z_valid U32 0/1 | U32 1 |
| 7 | U32 0 | extra28 I32 二补码 | U32 0 |
| 8 | U32 0 | 保留 previous reader56.word8；selected cold 为 0 且没有其他 writer | 保留 previous word8 |
| 9 | U32 1 | U32 1 | U32 1 |
| 10 | U32 1 | U32 1 | U32 1 |
| 11 | source-state U32 | source-state U32 | U32 3 |
| 12 | gain U32，selected 为 4096 | gain U32，selected 为 4096 | U32 0 |
| 13 | trim row flag U32 0/1 | trim row flag U32 0/1 | trim row flag U32 0/1 |

source-state 读取调用前同一 row 的 word11。mute 为 1 时写 3。unmuted 时写 `1+int(1<=old<=2)`。因此 cold 3→1，下一次 1→2，之后保持 2。direct 只能位于 row0；object 只能位于 row1..15。kind、component、XYZ、diffuse、z_valid、extra、mute、gain 或 row flag 越域时，materializer 拒绝并且不返回 next reader state。

连续向量的每个 reader callback 都提供完整 decoded-row 输入、16 条 previous reader56 hex 和 16 个 row flags。expected 提供完整 16 条 reader56 hex。fresh checker 可以独立重建 96 条 record，不读取 expected 作为输入。

## F05RT.6 target rows、shape和provenance

输出固定为16行，每行6个finite F32，顺序 `[L,R,C,LFE,Ls,Rs]`。row0的direct selector4只写LFE。row1..15使用 selected secondary8 几何和 main type2 panner，再把back6/7折到side4/5。

### F05R56P selected main type2 panner

普通功能参数见 `parameters/reader56-main-panner-v1.json`。secondary mask 为 bits0..7，primary mask 为 bits0..5，fold mode 为3。equal-power 使用 `quarter_pi=F32(word 0x3f490fdb)` 和 `half=F32(word 0x3f000000)`。几何由8×4 I32行生成；code3是负一sentinel并跳过。有效output slots固定为 `[0,1,2,4,5,6,7]`。

type1 direct 要求 word0=1、word1=4、word12=4096、word13=0、word11>=1。先建8项正零，只把slot3写1。type2 object要求word0=2、word12=4096、word13=0，word1/2/3各在0..32767。word4..11不参与当前flat panner数值；reader56 materializer仍须完整生成并保存这些词。

type2先令 `x=R32(word1/32768)`、`y=R32(word2/32768)`。每个y group独立算x权重。x小于等于组首时首点权重1；大于等于组末时末点权重1；否则找首个 `left_x<x<=right_x`，按 `fraction=R32(R32(x-left_x)/R32(right_x-left_x))` 计算equal-power pair。

selected type2 panner 的输入域要求word1/2/3按I32解释均在`0..32767`。F05L full-composition adapter必须在target调用前检查该域；word3<0专门映射为 `TARGET_NEGATIVE_Z`，以便区分writer leaf可编码的signed Z和当前target算法不支持的Z。此错误不返回target，不提交reader records或任何联合状态。不得通过忽略word3的当前数值贡献来放宽输入域。

y层按递增level处理。y小于等于最低层时最低组权重1；大于等于最高层时最高组权重1；否则找唯一 `low<y<=high` 区间，并用同样的fraction和equal-power pair。组内所有点共享该层权重。

equal-power中，fraction<=half时，`angle=R32(R32(R32(fraction×quarter_pi)×half)×4)`，返回 `(M32(cos(angle)),M32(sin(angle)))`。另一侧先算 `reduced=R32(1-fraction)`，使用相同angle并交换sin/cos。M32表示数学函数结果最近偶数到binary64，再转F32。每个乘、减和除分别R32，不使用FMA。

逐有效点按固定slot顺序写 `eight[slot]=R32(x_weight×y_weight)`。type1和type2随后都执行同一fold：先复制eight0..5到six0..5，再令 `six4=R32(six4+eight6)`、`six5=R32(six5+eight7)`。输出不归一化、不乘reader gain，也不消费component、z、diffuse、extra或word8。非法record返回 `R56T_PROFILE_UNSUPPORTED`，不产生部分target。

连续向量的96条reader record各自作为input-only panner case。clean panner生成576个target词，再与既有模型逐词postcompare，差异0。expected targets只用于计算完成后的比较。

每行target绑定三项身份：consumer envelope `g`、reader invocation `q=g−2`、delivered producer generation。target数组按row0..15连接为 `16×6 F32LE`。不能按目标内容相等合并行或选择旧generation。

本条输出可直接作为 F04C 的 `target_updates[g]`。receiver仍按自己的 `r=g−2` 计数；target producer不推进receiver gain rows。

## F05RT.7 cold、reset 和失败事务

cold构造三槽15行history、previous active全0、target snapshot的XYZ正零/ID `0xffffffff`/attribute0，以及16个cold reader56。callback_index和reader invocation都为0。

reset销毁全部 history generation、target snapshot、reader records和计数，再建cold。只清reader records或只清history都不构成reset。

执行顺序为：完整入口预检；marker preview；history preview；Type11 encode/decode preview；reader56 preview；16行target preview；最后一次提交全部next state。任何阶段失败都不返回正常targets，也不修改caller state。错误标签为 `R56T_STATE_ERROR`、`R56T_PIPELINE_RETIRED`、`R56T_CALLBACK_ORDER`、`R56T_INPUT_ERROR`、`R56T_PROFILE_UNSUPPORTED` 和 `R56T_NUMERIC_FAILURE`。失败后按I03退役，恢复需要完整cold reset。

该事务是独立实现合同。它不声明原应用能回滚已提交的上游selector、outer或component状态并继续。

## F05RT.8 连续向量与范围

连续向量从cold推进8个frontend callback。g0/g1只推进marker/history；g2..g7执行6次Type11/reader/target。g2覆盖7个marker−1和8个marker1，并交付混合generation1/2。g6前8行触发quiet-pair copy。g7前8行重新活动并立即交付generation7。

覆盖120行marker、6720 bytes marker record、6240 bytes control52、20160 bytes history payload、360个generation tag、96个reader56、576个target词和414 bytes Type11 wire。所有reader调用产生非零targets。reset重放与首次callback完全一致；7个拒绝用例保持输入state不变。

clean marker/history、reader56 materializer和selected main type2 panner均与既有Python模型逐项比较，差异0。Type11 wire leaf继续复用已有F05O/F05T定义。没有读取PCM、私密认证材料或调用SDK。真实整曲producer、非point Type11分支和无研究上下文fresh review仍开放。
