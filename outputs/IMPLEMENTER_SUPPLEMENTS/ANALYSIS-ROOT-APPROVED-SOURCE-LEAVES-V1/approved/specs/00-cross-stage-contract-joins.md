# CROSS_STAGE_JOIN_V1 selected-profile joins

状态：`J01_J04_LOCAL_STATIC_R2_PASS / WHOLE_EXECUTION_NOT_RUN`。本条只统一既有模块边界，不修改任何DSP公式，不依赖或交付厂商私有key。

1. F04RB→F05JL carrier唯一顺序为`[L,R,C,Ls,Rs]`；receiver lane映射`[0,1,2,4,5]`，所有QMF history、fit列与sparse channel ID保持该身份。codec的R/C交换只在F12 adapter发生。
2. 每source先且仅先执行一次`SharedAnalysisPrepare`推进F07；F06 consumer和F08/F09/F10/F11 consumer共享immutable receipt。任一下游失败丢弃analysis clone，禁止whole wrapper双推进。
3. source0/1照常完成四queue push/pop并提交队列/codec source状态；pop完成后立即分支，payload固定为`ABSENT_PUBLISHED_PAYLOAD_V1`，不得构造或解析EMDF，并跳过complexity、carrier及F12 payload验证。source1 popped JOC writer0标记`CONSUMED_SUPPRESSED`且不得延后给source2。source>=2才构造完整EMDF并执行这些验证。
4. durable write允许成功短写循环到4096；write/flush/readback fatal或append后root-swap前崩溃都abandon整个attempt。truncate/delete仅cleanup，恢复协议固定为`PROGRAMME_ORIGIN_FULL_RESTART`，即从programme输入起点重跑meter与main。

判别与故障向量见`test-vectors/CROSS-STAGE-CONTRACT-JOIN-VECTORS.json`。fresh r2 package-only独立复审登记见`test-vectors/CROSS-STAGE-CONTRACT-LOCAL-REVIEW-STATUS.json`：11个冻结文件身份通过，9/9 cases、61/61 expected-field assertions与3个连续cold queue maps通过，r1唯一P2关闭且无新增finding。因此J01～J04的local static contract为`YES`，当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`。真实EMDF字节构造/解析、DSP、F12 frame、物理durability/crash及4249-source cold replay属于交付后的独立实现执行，仍为 `NOT_RUN`；产品验收也仍为 `NOT_RUN`。
