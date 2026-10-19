# 测试策略

旧行为测试全部退役。新测试只使用独立批准的规范、公共事实和独立构造的输入；旧 expected、阈值、表和输出变化断言不具有权威。

当前 3 个 CLI 集成测试验证 help/version、编码拒绝且已有文件不变、旧命令拒绝。边界脚本检查当前 crate 依赖、算法根隔离与 domain 的 I/O 词法违规。它是工程防护，不是形式化的 Rust 语义证明。

权威规格已经选择 48 kHz、1,536 samples/AU 和 4,096 bytes/AU。后续测试按批准包新增：有界输入截断/溢出、PCM 字节与时钟会计、每 owner cold/reset/fatal、receipt 不变性、规范 wire 向量和独立外部解析/解码/最终输出比较。参数、阈值和表必须逐项绑定 `PARAMETER-MAP.md`，缺项则拒绝放行。

Direct-LFE 子阶段使用洗白后的 input-only `g0..g5` trace 和三个失败输入：从输入 recipe fresh 构造 1,536 个 F32，再执行 ring transition 后比较状态/输出；g2..g5 必须 exact-reuse g1 padded row。六回调 phase 标签是隔离 ring trace，不能用来重设生产 `g=0..4246` normal、`g=4247..4250` cached-drain 时钟。它只验证 ring 与 Type10 row0 seam，不代表 raw38 producer、whole callback、receiver/QMF 或产品验收。

P1.3-E4 assembly tests consume typed E2/E3 proposals for `g0..g5`: g0/g1 must produce Priming without a packet; g2..g5 must preserve row0 LFE bits/origin `g-2` and rows1..15 component bits/origin `g`. Mismatch cases cover callback/phase, programme/attempt/transaction, profile, provider lineage, stale spatial_audio before identity, missing row0, and ring-slot/origin disagreement. Dropping the assembly stage must leave committed state unchanged. Direct-LFE supplement rows and whole-chain Type10 hashes have distinct g1 identities and must not be cross-paired; E4 bit-copy assertions compare each assembled row to its consumed proposal input.

QMF_CORE direct-vector tests execute the nine-row fold/history, rotation, analysis scale/FFT/reorder, synthesis preformat/FFT/postmodulation/polyphase, and mode=1 24-slot delay/FIR/synthesis transitions. The washed analysis and synthesis literals are assertions after fresh computation; tests never use expected words or hashes as state seeds. The five-callback QMF and six-callback F04 whole-chain records are not run by this slice. Whole receiver routing, J05/parent integration, real ADM, and listening acceptance remain `NOT_RUN`.

Receiver mode=1 stage tests cover g0/g1 Priming, five distinct analysis lanes, surround-only mode=1 synthesis, warm history carry into a cached-drain callback, receiver count, and Type10/S09 phase and lineage rejection. The stage receives S09 as input from the F04R boundary; these tests do not execute F04R, same-runner F05R56 target production, selected-J05 projection, parent commit, or whole receiver output. Those executions remain `NOT_RUN`.

公共语法原语还必须覆盖 bit cursor 0/1/31/32/32768、容量溢出、CRC residual=0、单 bit 破坏、frmsiz 边界、header 保留码、非 48 kHz/非 5.1/dependent/非 4,096-byte 拒绝、六个 Huffman 表全部 symbol、两种 EMDF mode、四 payload 顺序、截断和失败零提交。

validation 的 structural parser、channel-map、topology、frame report 和 W04 与 production 分离。parser 成功不代表 codec conformance、音频正确或产品验收通过。

验证命令：fmt/check/clippy/test --locked 和 `python -X utf8 -B scripts/check-boundaries.py`。CI 在 Windows 和 Linux 运行。`INDEPENDENT_IMPLEMENTATION_VALIDATION` 和 `PRODUCT_ACCEPTANCE/W04` 仍为 `NOT_RUN`，不得由静态检查或单元测试替代。
