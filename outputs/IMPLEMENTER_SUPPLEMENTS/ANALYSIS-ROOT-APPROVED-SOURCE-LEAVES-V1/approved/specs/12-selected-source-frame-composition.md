# F07→F12 selected source-frame composition

状态：`OWNER_AND_PLAN_CONNECTION_DEFINED / REAL_FULL_SONG_FRESH_REPLAY_PENDING`。本条只组合已定义的 F07～F12 数值模块，不重复各叶算法。

## 1. 输入和时钟

source `s` 从 0 连续递增。输入为同代 `installed_audio[6][1536]`、同代 BSI、F13 payload、配置和 cold/previous state。外部顺序为 `[L,R,C,LFE,Ls,Rs]`，内部编码顺序为 `[L,C,R,Ls,Rs,LFE]`，映射 `[0,2,1,4,5,3]`。

source 0、1 完整计算和提交，但不发布。source `s>=2` 发布 AU `s-2`。EOF 不增加 codec flush。

input-only 向量必须逐 source 显式给出：`db_per_bit_codes[6]`、完整 BSI（dialnorm、RF compression、mixing、information）、六块 line DRC presence/code、`exponent_format`、JOC complexity，以及 JOC/OAMD/ID1/ID2 四种 payload 的 present/mode/bytes。缺席也是规范值，写成 `present=false,mode=ABSENT,bytes_hex=""`，不能依赖 builder 默认值。audio recipe 也属于输入，固定使用 UTF-8 canonical JSON：key 按字典序、分隔符为逗号和冒号、不允许非有限数。`audio_recipe_sha256` 直接覆盖该输入对象。

## 2. 每 source 固定顺序

1. F07 input history 按每块读取 old history[256:512]、左移256并追加当前256样本的规则执行；cold source 的首个非零 delayed sample 位于 index1024，成功后保存 installed audio 的末1280样本。
2. F07 transient 只消费 installed audio，按 6 blocks×5 FBW 生成 short/other consumed、structure refresh 和 staged transient state。
3. F08 对 delayed audio 执行一次 primary/companion 变换。五个 FBW 和 LFE 的两个 bank 各自拥有 128-word history。FBW 两 bank 使用同一个 short_consumed；LFE 不使用 FBW short flag。
4. 从这一次固定谱生成 POWER snapshot、PRIMARY raw exponents、gain/db、F09 refresh/strategy 和 F10 normalized anchor/reuse plan。
5. F11 只读固定 POWER、structure flags 和 converter 自有输入，独立计算 main/converter。r2 合成验收的精确规则为 `main[i]=(5*s+i) mod 32`、`converter[i]=(16+5*s+i) mod 32`、`converter_snr_offset=(341+73*s) mod 1024`；三个结果均由 source `s` 计算，输入向量不供应 expected code。一般产品 producer 仍以 F11 叶规范为准。
6. 用 short snapshot 和 old previous_short 一次生成全帧 30 个 dither flags 与 staged next_previous_short。
7. F12 candidate search 只读固定 spectra、expanded exponents、gain ranges、db codes、converter plan、BSI 和 payload。每次 trial 新建 BAP、mantissa groupers 和 bitwriter。
8. 最后一个成功 candidate 成为 best。完整 4096-byte frame、CRC 和所有字段成功后，统一提交 staged state一次。

## 3. Transform owner

同一 source 只能推进 primary/companion/LFE histories 一次。计划阶段和编码阶段共享同一计算所得 spectra 与 next histories。禁止在同一 state 上为 F12 再执行一次 F08。

允许实现选择另一种等价结构：计划和编码各从同一帧前 state 克隆并重算，但只能提交一份 next state，且两次结果必须逐词相同。共享一次谱结果是规范主路径。

F06 none-DRC secondary reconstruction 是独立 owner。它拥有自己的 forward history 和 inverse carry，不能借用 F08 companion state。

## 4. Plan 的固定内容

每个 source plan 至少含：

- short_consumed、other_consumed、structure refresh，各 `6×5`；
- primary/companion FBW spectra，各 `6×5×256`；
- primary/companion LFE inputs及七项输出，各6块；
- POWER snapshot `6×5×217`；
- PRIMARY raw exponent rows `6×5×217` 和 LFE `6×7`；
- main strategy、frame-code或逐块格式、normalized anchors/groups、expanded BAP exponent 和 gain range；
- six db-per-bit codes；
- five converter frame codes 与 converter SNR offset；
- staged transform histories 和 staged next_previous_short。

F09/F10 只执行一次。Candidate q 改变 BAP、mantissa 和容量，不改变 plan。

## 5. Trial/search 与提交

先试 q=16。随后 low=16、high=784；仅当 high-low>1 且 midpoint计数<10时继续。每次取 floor((low+high)/2)：成功保存best并更新low，失败更新high。初始16不计入midpoint计数。all-fail序列为[16,400,208,112,64,40,28,22,19,17]，区间到[16,17]后停止，共10个真实trial。

Candidate failure 包括容量、BAP、量化 sentinel、分组或字段构造不可行。它只销毁本次 scratch，不提交 transform history、previous_short、plan state、payload FIFO 或 source clock。下一个 candidate 重用同一固定 plan。

若没有 candidate 成功，source fatal，调用前所有 owner state 保持，programme retired。若 best 成功，只提交一次 source next state。不能在 failed candidate 后把部分 BAP 或 bit cursor带入下一次。

## 6. 5 FBW、LFE 和短块

每帧6块。五路 FBW 各217 bins；LFE各7 bins。每块 mantissa 遍历顺序 `[L,C,R,Ls,Rs,LFE]`，随后 bin递增。三个 grouped BAP accumulator 每块重建，不跨块。

short flags 为严格 I1，1 选择 short。两 bank 同时选择同一 long/short branch。`short_syntax` 只看当前30项；`previous_short` 只参与 dither。source0/1 的 short/history 同样提交。

## 7. Reset、EOF 和错误

入口先验证 provider身份、state/source generation、6×1536 finite input、30 short I1、谱/指数/plan shape、BSI 和 payload。失败在任何数值工作前拒绝。

reset 重建 input history、transient histories/queues、primary/companion/LFE histories、gain/db/policy state、codec transform state、previous_short和source index。cold/reset canonical state 必须逐字段相等；从 reset 重放 source0 的 core output 与首次 cold run 相同。发布开始时不 reset。

owner digest 禁止使用 generation projection。`F07_F12_OWNER_STATE_V5` 对完整 owner 值求 canonical SHA-256：input history 7680 U32；transient 的20+30+90+5个F32 words及两个20项I1 queue；primary/companion/LFE四组640+640+128+128 history words；policy的64 U32和两组30 U16；codec的256 U32、18 U16及bit cursor；完整 previous_short、payload FIFO、publication count及 lifecycle。每条向量保存完整 `before_state` 和 `after_state`。fatal 在 staged 数值工作和10次真实trial之后发生，但除 lifecycle 外的完整 owner before/after digest 必须全部相等。

EOF 只结束实际4249 source extent，不补 transient、transform、exponent、converter 或 codec 私有帧。

## 8. 验收范围

非零连续向量含3个成功source和1个all-candidate-fail source。每个 source 的谱由 `x[s,c,k]=(-1)^((s+c+k)&1)*2^-16` 精确生成，FBW各217项、LFE 7项，六块复用同一固定谱；`POWER=x*x` 按binary64 round-to-nearest-even，展开指数为15，FBW plan 为 `strategy=[1,0,0,0,0,0], base=15, packed_groups=72*62, gain=0`，LFE为 `strategy=[1,0,0,0,0,0], base=15, packed_groups=[62,62]`。F12使用public BAP表，真实执行BAP、mantissa量化/分组、scratch bitwriter、4096-byte frame和CRC；fatal以512-bit合成容量执行10个全失败真实trial。覆盖mixed short、previous_short carry、独立main/converter codes与SNR offset、source0/1 suppression、source2→AU0、fatal全owner digest和cold/reset/replay。向量不含PCM。

真实全曲 fresh replay、F08新数学参数的编码/render质量、LFE gate非零配置和极值输入仍开放。


## 9. r4 数值状态来源

r4 的 29,043 个被审查 U32 state words 均由 explicit audio recipe 和已交付参数计算。F07 input history 使用上述 1024-delay/last1280 规则；transient 的 two/three/stats/mean/queue 逐块执行 F07 方程；F08 四组 next history 由最后 delayed block、window 和 fold 方程产生；policy 64词取 audio-derived delayed samples 的 F32平方；codec 256词取同代 delayed block 的 F32 words。禁止 generation 线性 marker 或与 audio 无关的填充值。outer composition 的 best_q 和 trials 直接引用同一次真实 BAP/mantissa/frame search；三个成功 source 均为 q422。


## 10. r5 owner 投影与 replay

policy 的64词固定取 external LFE（输入channel index3）的 delayed_audio[3][1472:1536]，按sample index递增逐项先平方、再单次RNE到binary32，最后写对应U32 bit pattern。codec 的256词固定取 external L（输入channel index0）的 delayed_audio[0][1280:1536]，按sample index递增写binary32 U32 bit pattern。禁止跨channel flatten后再切片。

成功时input/transient/transform/policy generation在staged数值工作各推进一次；codec generation仅在完整frame成功后推进一次，cursor置32768。all-fail的staged state已含本source的input/transient/transform/policy及codec sample words，但staged codec generation和cursor保留committed值；fatal commit仅改变lifecycle。published source成功时FIFO追加 {au:source-2,payloads:input payloads}，suppressed/fatal不追加。

搜索先试q16，再仅在 high-low>1 时执行最多10次midpoint。all-fail序列为 [16,400,208,112,64,40,28,22,19,17]，共10 trials，不重复末尾q16。成功source仍各有11 trials，best q422。cold replay core projection精确定义为 events,dither,best_q,trials,published,au,source_commits,owner_after_sha256；first和replay分别canonical hash该object，并要求两hash及完整after state都相等。


## 11. mantissa fields 摘要

mantissa_fields_sha256 不是 opaque metadata。先从 complete-frame scratch BitWriter.fields 按原 append 顺序筛选 label 含字面量 .mantissa[ 的条目。label grammar 固定为 lock[b].mantissa[i]，其中 b 是十进制 block index 0..5，i 是该 block 内按写入顺序扁平化后的 mantissa item index（包含量化为跳过或grouped的item位置）。每个条目必须是只含 label,begin,end,value,width,semantic 的 JSON object：label 为 UTF-8 string；begin 为从完整 frame bit0 起的 inclusive offset；end 为 exclusive offset；value、width 为整数；semantic 为 bool。名称和 offset 都进入摘要。外层为 order-sensitive JSON list，不排序条目。序列化固定为 UTF-8 canonical JSON：object keys 按字典序、separators 为逗号和冒号、无空白，list 顺序保持。对这些 bytes 计算 SHA-256，以 lowercase hex 记录。

## F12SC.10 GTC shared-analysis consumer入口

GTC中F12SC只接受`SharedAnalysisReceipt`及其派生`NumericPlanReceipt`。adapter必须核验二者source/generation/provider digest一致且`advancement_count=1`；不得再次调用F07 history/transient。source0/1在F13 queue pop后直接取得`ABSENT_PUBLISHED_PAYLOAD_V1`，上游不得构造或解析EMDF；F12仍执行数值计划、DRC和codec state推进，但跳过EMDF/complexity/carrier及全部payload字段验证。source>=2才接受完整EMDF并执行payload验证。
