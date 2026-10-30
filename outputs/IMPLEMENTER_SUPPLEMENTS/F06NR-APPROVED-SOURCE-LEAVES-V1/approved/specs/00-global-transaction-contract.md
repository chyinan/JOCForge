# Selected-profile 顶层事务合同（clean draft）

状态：`PASS_STATIC_CONTRACT_EVIDENCE_GAPS_REMAIN`。r2 package-only静态独立复审已通过；local executable transaction与whole execution仍为NO。范围只覆盖固定 48 kHz、1024 kbps、5.1、六块、uncoupled、no-SPX、no-AHT、keyless checksum 路线。

本合同组合主包已经定义的局部 owner 和事务。它不声称原生实现具有全局 rollback，也不要求在 fatal failure 后继续。新实现必须在自己的隔离 state root 上 stage 可复制状态；任何 fatal failure 都使 programme 进入 `RETIRED`，丢弃本 attempt 的临时输出，恢复只能从 programme 输入起点完整 reset/restart。

## 1. 基本对象

### 1.1 ProgrammeState

一个 programme attempt 只拥有一个 root state。所有公开入口都接收并返回这个 root 的 generation；模块不得把隐藏 singleton、文件名、内容 hash 或上一 attempt 的状态当作 owner。

顶层状态：

- `programme_id`：输入节目身份；reset 后保持同一输入身份。
- `attempt`：从 0 开始；每次 fatal 后完整 restart 加 1。
- `status`：`METERING | READY | ACTIVE | RETIRED | FINALIZING | DONE`。
- `phase_generation`：每次成功顶层 commit 加 1。
- clocks：meter callback/window、main callback、codec source、JOC dispatch/writer、published AU。
- `measurement_receipt`：完整 meter prepass 成功后不可变。
- `owners`：表 1 的 committed owner roots。
- `output_ledger`：programme 临时 EC3、已 durable 的 AU 数/长度/末 AU hash。
- `eof_ledger`：输入尾部、两个 drain 域、source/AU 数和 pending payload disposition。
- `fatal`：仅 `RETIRED` 存在，记录 stage、error、transaction 和 commit visibility。

状态转换仅允许：

```text
METERING -> READY -> ACTIVE -> FINALIZING -> DONE
    |          |        |            |
    +----------+--------+------------+-> RETIRED
RETIRED --full restart--> METERING (attempt+1)
```

`DONE` 和 `RETIRED` 都拒绝普通 callback/source/module 调用。`DONE` 只允许读取结果；`RETIRED` 只允许 `full_restart` 或销毁。

### 1.2 Owner roots

| owner_id | 持久内容 | generation 单位 |
|---|---|---|
| `reader_eof` | reader cursor、last cached input、normal/drain counters、EOF phase | main callback |
| `frontend_control` | F01 config/DBMD、event sampler/cache、control snapshot、callback clock | main callback |
| `spatial_control` | F03 feature48、target/current/previous、active/marker、metadata history | main callback |
| `spatial_audio` | F04 PCM/history/filter/component/receiver/QMF/ring/temporal | main callback |
| `joc` | F05JL receiver/fitting QMF、temporal、geometry、fit generation、sequence、writer | JOC dispatch |
| `meter` | independent meter/OAR/K-filter/window/integrated/true-peak state | meter callback/energy record |
| `drc` | F06 secondary、statistics、scheduler、override FIFO | codec source |
| `analysis` | F07 input/transient queues、plan F08 histories、gain/db、F09/F10 state | codec source |
| `codec` | F12 transform histories、previous_short、source clock | codec source |
| `metadata` | F13 timer、四 FIFO、writer/pop generations、fixed mode | codec source |
| `integrity` | selected F14K is stateless; only current-frame checksum receipt | frame |
| `output` | temporary output identity、durable AU ledger、finalization state | published AU |

每个 staged/committed owner receipt 必须含：owner_id、before_generation/hash、after_generation/hash、changed、commit_count、transaction_id。Hash 只能验证 state identity，不能替代完整 state bytes；实现必须拥有可提交的 after state。

### 1.3 不可回滚边界

本合同不对以下外部行为承诺 rollback：源文件读取、操作系统 I/O、已 durable 写入的 programme 临时文件、诊断日志。处理方式是：

- 输入 reader 位置属于 committed state；prepare 通过显式 offset 读取，不依赖不可恢复的隐式流位置。
- AU 只写 programme 私有临时文件。fatal 后整个 temp artifact 标记 abandoned，不作为产品输出，不在同 attempt 继续。
- 日志只能在 receipt commit/abort 后写，不是运行输入。
- 最终产品路径只在 `DONE` transition 中通过 atomic replace 暴露；失败 attempt 永不覆盖此前成功产品。

因此没有“写出后回滚一个 AU并继续”的路线；只有完整 attempt restart。

## 2. MeasurementTransaction

Meter 是独立 phase，不与 main owner 混用。

1. `METERING` 从全部 meter owners cold 和 meter callback 0 开始。
2. 每个 meter callback 在 clone 上完成 F06OF/F06OC、K-filter、window、integrated 和 true-peak next state；成功才替换 meter root。
3. Meter EOF 使用自己的规则：normal 4247、三个新建零输入 DRAIN、完整1024前缀与960窗口规则；不借用 main 的四 cached callbacks。
4. 完成后校验 meter ledger，生成 immutable `MeasurementReceipt`：programme/config/input identity、dialogue、ITU、true peak、gating kind、selected LUFS、terminal meter hash和计数。
5. 一次顶层 commit 将 status `METERING -> READY`。随后构造全新的 main owner cold root；meter working state不作为 main state seed。
6. meter 任一 fatal：discard staged callback，status→`RETIRED`，无 MeasurementReceipt。

为了避免推断旧 terminal meter 是否可复用，`full_restart` 一律丢弃 receipt并从 `METERING` 重跑。这是保守 restart 规则，不是 native rollback 声明。

## 3. CallbackTransaction

### 3.1 输入与映射

`callback_index g` 从0连续递增。主路径总 callback 为4251：4247 normal +4 cached。codec source `s=g-2` 仅在 `g>=2` 存在。JOC dispatch `d=s+1=g-1`。

### 3.2 Prepare 顺序

对 committed root 建立 `CallbackTransaction(g)`，所有步骤只写 stage root：

1. 验证 status=`READY`（仅 g=0）或 `ACTIVE`、g连续、phase和输入范围。
2. F01/F02 准备 PCM、控制、padding/drain 输入与 reader/EOF next。
3. F03 feature、selector、U04、outer、target proposal、marker和metadata-history proposal。
4. F04 audio/component/receiver/QMF 全部 next；preview不得把 next 当第二次输入。
5. 若 g<2：验证无 codec source/JOC dispatch，转到 callback-only commit。
6. 若 g>=2：创建嵌套 `SourceTransaction(s,d)`，使用本 callback stage 的同代输出。

F03G 的“提交”在本合同中解释为 proposed target owner receipt；真正 persistent commit 在 CallbackTransaction 根替换时发生。F04 receiver/QMF 若被 F05JL消费，必须通过同 transaction 的 before/after owner token；不得分别写同一 committed buffer。

上述投影的权威逐字段拆分见 `specs/00-selected-owner-projection-and-j05.md` 与 `parameters/selected-owner-projection-v1.json`：F04C唯一执行J05 receiver/QMF，F05JL只消费投影；F05R56唯一推进reader/target；F13唯一推进四queue；F15只协调leaf receipts与durable commit。任何旧文档中重叠的“拥有”表述都由该合同覆盖。

owner projection r6独立静态复审130/130通过，G01–G03在静态合同范围均为`CLOSED_LOCAL_STATIC`，登记见 `test-vectors/OWNER-PROJECTION-LOCAL-REVIEW-STATUS.json`。三阶段模型中该结果解除pre-handoff static blocker；whole execution仍为post-handoff `NOT_RUN`，不反向把已经闭合的静态合同标回未就绪。

### 3.3 Callback-only commit

g=0,1 没有 codec source。所有 F01–F04 步骤成功后：

- 校验 owner after generations 都为 g+1；
- 原子替换 reader_eof、frontend_control、spatial_control、spatial_audio roots；
- main callback clock=g+1，phase_generation+1，status=`ACTIVE`；
- commit_count=1。

### 3.4 Callback abort

任一步 fatal：discard整个stage root；committed owner roots不变；写 fatal receipt后status=`RETIRED`。不运行后续步骤，不发布任何 callback/source/AU结果，不允许 g 重试。

## 4. SourceTransaction

SourceTransaction 是 CallbackTransaction 的嵌套 stage；只有它成功，父 callback 才能 commit。

### 4.1 Prepare 顺序

1. 验证 `s=g-2`、`d=s+1`、所有输入 generation 和 owner token。
2. **SharedAnalysisPrepare**：仅一次推进F07 input history/transient，生成immutable `SharedAnalysisReceipt`，其中含same-source installed/delayed audio、30个short flags、primary未量化spectrum、analysis before/after generation/hash；after state只留在analysis clone。
3. F06 consumer读取同一receipt的primary、short flags和installed audio，在独立drc clone运行F06 forward/inverse/secondary及statistics/scheduler，生成BSI/DRC。它不得调用F07，也不得提交analysis clone。
4. F08/F09/F10/F11 consumer读取同一receipt，在同一个analysis clone继续六块计划并形成`NumericPlanReceipt`；F12SC若被调用只能采用consumer入口，禁止其whole-composer入口再次推进F07。F06或任一后续失败均丢弃包括F07 after在内的全部stage roots。
5. F05JL：在 joc clone 上生成当前 ID14 writer，所有内部 retry均scratch；完整 body 后得到 JOC owner receipt，尚不写 committed root。
6. F13 producer barrier：准备 ID11、ID1 timer/body、ID2、ID14 四body，全部携带本 writer generation。timer next仍在 metadata clone。
7. F13 四queue联合事务：复制四queue，在 clone 上严格 ID11→ID1→ID2→ID14 各 push 后 pop；source0/1也完整执行并处置popped body；任一步失败丢弃全部 queue clone。
8. publication payload gate：若`s<2`，设置`publish=false`和`payload_absent=true`，跳过EMDF序列化、complexity/carrier预算及published payload验证，并向F12传入规范`ABSENT_PUBLISHED_PAYLOAD_V1`；尤其source1弹出的JOC writer0被消费并丢弃，不得转入后帧。若`s>=2`，才按ID11、ID14、ID2、ID1组装EMDF并验证complexity、mode、checksum和carrier预算。
9. F12：从 codec clone、NumericPlanReceipt、BSI/DRC和上一步的EMDF或absent-payload运行所有candidate；candidate失败不改任何stage owner。成功生成完整4096-byte frame、CRC receipt与codec next。
10. publication gate：s<2保持不追加AU；s>=2要求`au_index=s-2`与ledger连续并执行第5节durable temporary append。

### 4.2 NumericPlanReceipt

`SharedAnalysisReceipt`必须包含source、installed/delayed域身份、analysis before/after generation/hash、F07 advancement_count=1、30个short flags和primary未量化spectrum identity。`NumericPlanReceipt`引用该receipt的canonical digest，并包含六块完成标志、short/other、primary/companion/LFE spectra identity、POWER/PRIMARY计划、gain/db、normalization、converter结果和provider identities。F06和F08消费者必须引用同一个digest。

F07历史/transient推进只发生一次；F09G逐block推进只发生在同一analysis clone。F06 secondary/DRC失败或六块中任一失败时committed analysis root完全不变。F12只读取receipt；不得重新运行或二次推进F07–F10。若whole F06WH/F12SC wrapper都企图拥有F07，adapter必须拒绝`ANALYSIS_DOUBLE_ADVANCE`。

### 4.3 F13 FourQueueReceipt

必须包含四queue的 before/after generation/hash、每个push slot/pop writer、publication order和一个总 `commit_count`。四queue只有全部操作与EMDF输入验证成功后才可成为 metadata after state；不允许前三queue已提交、第四queue失败。

### 4.4 成功提交

所有 prepare、frame 和必要 durable append 成功后，父 CallbackTransaction 以一次 root swap 提交：

- F01–F04 callback owners；
- joc、drc、analysis、codec、metadata owner after states；
- callback/source/dispatch/writer clocks；
- publish=true时的 published AU ledger；
- phase_generation+1。

所有 owner receipt 的 commit_count 必须为1或changed=false时0；顶层 transaction commit_count必须恰为1。source0/1同样提交全部状态，只是不增加published count。

### 4.5 Abort 与 fail-stop

任何 fatal（producer、generation、queue、no-best、CRC、append）执行：

1. 停止后续stage；
2. 丢弃所有内存stage roots；
3. 若临时输出已写入，标记整个attempt artifact abandoned；不截断后继续；
4. committed root除global status/fatal receipt外保持调用前identity；
5. status→`RETIRED`；
6. 拒绝所有后续callback/source/module调用；
7. 仅允许完整 `full_restart`。

这是一条新实现的隔离事务合同。若某 adapter 无法在 clone 上运行并会修改外部/隐藏状态，则该 adapter 不满足本selected-profile接口，不能以“最终会fail-stop”掩盖未知提交。

## 5. AU durable write

每个attempt使用唯一 temp output，不直接写最终产品路径。

对于publish=true：

1. 预先验证 frame恰4096 bytes、AU index连续、CRC residual=0、frame hash已固定。
2. append到temp文件的预期offset=`au_index*4096`；禁止隐式append到未知位置。
3. 使用显式offset执行完整append loop。一次成功write可返回1..remaining bytes；持续写到累计4096。返回0、错误或写过界为fatal。短写本身不是fatal，也不提交ledger。
4. 写满后检查temp长度和4096-byte读回hash，flush/fsync使本次append durable。
5. 返回 `DurableAppendReceipt`，然后才允许顶层root swap增加published clock。

append前失败：temp和committed state都不变。append loop中的write错误/零进展、写满后的readback或durability失败：attempt立即RETIRED，temp abandoned，committed state不变；可截断到旧committed length仅作为关闭/删除前的cleanup，不得据此在同attempt恢复。append成功但进程在root swap前崩溃：该attempt也不得resume；启动时依据未完成attempt marker丢弃temp并从programme起点完整restart。未知尾部允许cleanup truncate，但trusted ledger绝不是恢复authority。

## 6. RETIRED、restart 与 reset

`full_restart` 前必须：

- 关闭并标记abandoned temp output；
- 丢弃MeasurementReceipt、所有 pending queue/body、diagnostic scratch；
- 销毁整个 owner root，不逐字段修补；
- attempt+1，phase_generation=0，所有clock回cold；
- 从同一输入programme identity重新进入METERING。

冷构造必须由 `test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json` 的owner清单和各模块规范共同确定。若任一owner缺cold constructor/hash，restart失败且保持RETIRED；不能用零填未知state。

## 7. EOF、pending ledger 与 DONE

### 7.1 独立 meter EOF

- normal callback=4247；
- F06OF new-zero DRAIN=3；
- 完整1024前缀=6,522,880 samples；
- 960-sample energy windows=6794；
- 已消费但未提交energy window samples=640；
- measurement commit一次后进入READY。

### 7.2 Main EOF

- normal callback=4247；最后1050 real+486 zero padding；
- cached callbacks=4，复用最后主输入/control snapshot并推进main state；
- total main callbacks=4251；
- valid codec sources=4249；
- JOC dispatch/writers=4249；
- published AUs=4247；每个4096 bytes；
- 不增加F07/F09/F12/JOC/metadata/DRC专属flush。

### 7.3 Pending payload

最后source后预期：普通ID11/ID1/ID2 writers 4247和4248 pending；ID14 writer4248 pending。它们不进入新AU。FINALIZING时记录每个pending writer id、payload hash和disposition=`DROP_AT_DONE`，然后释放queue storage；不调用pop推进clock。

### 7.4 Finalize

进入FINALIZING后检查完整ledger与所有terminal owner hashes，temp长度必须 `4247*4096=17,395,712`。fsync并原子replace到最终路径；成功后写final output identity并status→DONE。replace失败则RETIRED，旧成功产品（若有）保持不变，本attempt temp abandoned。

`eof_ledger` 是运行时累计账本，不是构造时已经等于终值的常量对象。`METERING/READY/ACTIVE/RETIRED/FINALIZING` 中的 normal/cached/total/source/dispatch/published 字段使用schema规定的范围，并在同attempt的committed snapshots间单调不减；last real/padding在最终normal callback前为0，提交该callback后固定为1050/486。只有status=`DONE`时，schema的`if/then`才要求全部九个终态标量、4247个durable AU/17,395,712 bytes以及pending disposition=`DROP_AT_DONE`。跨snapshot单调关系由运行时validator检查，不能只靠单snapshot JSON Schema冒充时序证明。

## 8. 错误分类

- `INPUT_OR_GENERATION_ERROR`：prepare前拒绝，无模块stage执行。
- `STAGE_FATAL`：模块或provider数值/shape/state失败，discard stage并RETIRED。
- `CANDIDATE_INFEASIBLE`：仅F05/F12内部candidate scratch失败；有后续合法candidate时不是fatal。
- `NO_ACCEPTED_CANDIDATE`：stage fatal，RETIRED。
- `QUEUE_TRANSACTION_ERROR`：四queue clone失败，四committed queue不变，RETIRED。
- `DURABLE_OUTPUT_ERROR`：temp abandoned，root不提交，RETIRED。
- `POST_TERMINAL_CALL`：RETIRED/DONE普通入口固定拒绝且任何state不变。

错误标签不冒充native错误码。所有 fatal receipt 必须记录 first failing stage；后续stage状态为NOT_RUN。

## 9. 验收

`test-vectors/GLOBAL-TRANSACTION-INPUT-VECTORS.json` 是状态机与owner receipt的input-only连续/故障注入向量。实现先计算实际结果，再与expected比较。最低验收：

- cold meter→READY→前五main callback连续路径；
- source0/1提交但不发布，source2发布AU0；
- F03、F04/F05、F09各block、F13四queue、F12 no-best和output short-write注入；
- 每个fatal后post-call拒绝和full restart；
- append成功/root-swap前崩溃按restart处理；
-完整EOF ledger、pending payload记录、temp最终长度与DONE；
-所有abort case committed owner hashes不变，唯一允许变化是global status/fatal receipt。

另有3个schema回归投影：ACTIVE prefix ledger必须合法且不触发DONE终值约束；完整DONE ledger必须合法；把同一prefix伪装为DONE必须因`if/then`终值及output ledger约束而拒绝。schema单snapshot验证之外，运行时validator仍须执行同attempt单调关系。

## 10. 不在本合同内关闭的证据

本合同只定义事务与owner连接，不补齐缺失算法/参数，不证明native相同提交，不运行实际ADM/JOC/render。具体剩余项见 `specs/00-global-transaction-evidence-gaps.md`。

r2 package-only 独立静态复审登记见 `test-vectors/GLOBAL-TRANSACTION-CONTRACT-LOCAL-REVIEW-STATUS.json`：schema v2 的 57 个引用全部解析，12 个 owner、7 项 runtime invariant、DONE-only 九个终值、3 个 schema regression 和 14 个顶层事务 case／35 个展开检查全部一致。r1 的两项问题已经关闭，且没有新增问题。owner projection r6 进一步以 130／130 检查把 G01～G03 标为 `CLOSED_LOCAL_STATIC`。G04～G11 属于交付后的 whole execution，当前为 `NOT_RUN`，不反向阻断 PRE_HANDOFF。`global_rollback=false` 与 fatal 后 programme-origin full restart 保持不变。

## 11. J01–J04 规范连接版本

本合同采用`CROSS_STAGE_JOIN_V1`：carrier顺序`[L,R,C,Ls,Rs]`；每source一次SharedAnalysisPrepare供F06/F08消费者共享；source0/1完成四queue push/pop但使用absent published payload；durable partial write循环到4096，任一fatal均abandon attempt并programme-origin full restart，truncate只作cleanup。规范判别向量见`test-vectors/CROSS-STAGE-CONTRACT-JOIN-VECTORS.json`。fresh r2独立静态复审以9个case、61项字段断言和3个连续queue map关闭J01～J04及r1唯一P2，登记见`test-vectors/CROSS-STAGE-CONTRACT-LOCAL-REVIEW-STATUS.json`。cross-stage local static contract=`YES`，当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`；whole transaction 的实际执行与产品验收分别属于下游阶段，仍为 `NOT_RUN`。
