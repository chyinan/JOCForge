---
status: provisional_implementation_contract
topic: "Flat7x D0 four-channel E-AC-3 producer"
spec: "ETSI TS 102 366 V1.4.1; ETSI TS 103 420 V1.2.1"
confidence: "wire syntax is standard-bound; source mapping and encoder choices are provisional"
tags: [JOCForge, Flat7x, E-AC-3, D0, four-channel]
audience: implementer
---

# Flat7x D0 四声道编码契约

## 目标

实现一个 Flat7x 专用 D0 音频帧生产器。I0 和现有 5.1 六档路径保持原样。D0 的 source-role map 继续使用可替换的临时方案，并随同一来源 receipt 传递。

目标帧为 48 kHz、strmtyp=1、substreamid=0、acmod=5、lfeon=0、bsid=16、chanmap=0x1A00、6 个音频块、4,096 字节。frmsiz=2047。D0 编码声道顺序是 Ls、Rs、Lrs、Rrs。JOC 重建仍是 7 个 carrier，ADM 行数、D0 coded channel 数和 JOC carrier 数分别保留。

## 音频块规则

ETSI 将一个 audio block 定义为 512 个样本：前 256 个来自上一个 block，后 256 个是新样本。512 点 TDAC 产生 256 个频带系数。六个块共消费 1,536 个新样本。

v1 采用可替换的首版编码策略：

- 每个 256 样本块使用 256 个历史样本和 256 个新样本，执行 ETSI 窗函数和 512 点长变换。
- 6 个块都用长变换。blkswe=0。
- D0 不使用 coupling、AHT、spectral extension 或 transient pre-noise processing。
- expstre=1。每个 block、每个 coded channel 显式写 D15 exponent strategy（2-bit code 01）。
- chbwcod=60，按 ETSI 公式得到 endmant=253。D15 每个 channel/block 写 1 个 4-bit U4 seed 和 84 个 7-bit U7 group。
- U7 将 3 个有符号 exponent difference 先各加 2，再编码为 25×m0 + 5×m1 + m2。无差值的 0,0,0 组编码为 62。
- gainrng 按 ETSI Table 6.33a 从 512 样本分析窗口的最大绝对值计算。
- snroffststr=0 使用 SNR strategy 1。v1 的 coarse/fine offset 设为 0/0。bamode=0 使用 ETSI 定义的默认 bit-allocation 参数。
- D0 采用四声道共享 bit pool。BAP 必须遵守 ETSI Clause 6.2。mantissa 按 Clause 6.3 编码。

D0 的 active-bin extent 是 253 bins/channel、4 channels、无 LFE。当前公开 provider 只覆盖 217 full-band bins 加 LFE 7，形状不兼容。新增 D0 专用 4×253 provider，从公开 ETSI TS 102 366 V1.4.1 Clause 6.2.3 和 Tables 6.6～6.12、6.14～6.16 实现。不要扩大或改写 5.1 provider，也不要复制内部 CODEC-PUBLIC-BAP-PROVIDER-V1 的数据。

当前公开 BAP provider 的 profile 是 48 kHz ordinary full-band 217 bins 加 LFE 7，不能覆盖 D0 的 253 bins、4 channel、无 LFE。D0 必须使用独立的 4×253 provider；只从公开 ETSI TS 102 366 V1.4.1 Clause 6.2.3 与 Tables 6.6～6.12、6.14～6.16 实现。不要扩大或改写 5.1 provider，也不要复制内部 CODEC-PUBLIC-BAP-PROVIDER-V1 数据。BAP 1、2、4 分别按 3 个、3 个、2 个 mantissa 分组；BAP 3 的合法码值是 0～6；BAP 0 配 dithflag=0 产生零值。
- dithflage=1 且所有 block/channel 的 dithflag=0。frmfgaincode=0，dbaflde=0，spxattene=0。
- BSI 的 dialnorm、compr 和 mixing fields 由现有同源 F06ND/F12 policy 输入。缺少 receipt 时返回 typed error，不在 D0 writer 内另造 DRC 值。

这些开关是首版 JOCForge 策略。它们只选用 ETSI 允许的语法分支，不表示 DEE 的音频参数或听感一致。若 4,096 字节目标帧装不下，返回 bit-budget error；不要截断 exponent、mantissa 或 EMDF。

## 帧尾、EMDF 和 CRC

现有 D0 frame assembler 要求 BSI、6 个 audblk、EMDF skip field、帧尾和 CRC2 的总长恰为 32,768 bit。新增显式 frame-level aux-data span。ETSI 的 auxbits 位于所有 audio block 之后、CRC2 之前。首版将未用余量编码为零，并设 auxdatae=0。不要把这些位拼入 block 5 的 mantissa span。

skipflde=1 后，每个 block 都按语法读取 skiple。只有 zero-based block index 2 设置 skiple=1，其余 5 个 block 设置 skiple=0。block 2 的唯一 skip field 承载 F13 返回的完整 EMDF 容器，长度必须为 1～511 字节。512 字节或更长、缺失容器或额外 skip field 都在 F15 前拒绝。EMDF 的实际字节仍由现有 F13 owner 生成；本契约不复制参考 payload。

D0 组帧使用已审查的 frame assembler：frame 为 4,096 字节，CRC2 占末尾 16 bit，CRC residual 必须为 0。D0 不得改写 I0 frame。

## 状态和 receipt

新增 D0 专属 Flat7xD0EncoderState，按 programme、attempt、source identity 和 Flat7xSourceGeneration 建立。它持有 4 个 channel 各 256 个历史样本，并分别记录 reset cause、next source sample、AU index。SharedAnalysisIdentity、analysis generation、Flat7xSourceGeneration、F12 generation 和 F13 owner generation 是独立字段，不能相互替代。

冷启动、seek、source discontinuity、sample-rate 变化或 role-map 变化时，用零历史开新 generation。每个 AU 处理 6×256 个新样本。最后一帧有 1～1,535 个有效样本时，用零补至 1,536，并在 receipt 记录 valid_samples 与 padded_samples。长度为 0 时不再输出 AU。原始 EC-3 会解码到完整 1,536 样本，首版不提供 sample-accurate tail trim。 I0 和 D0 receipt 必须提交相同的 valid_samples/padded_samples；计数不同就拒绝整 AU。

状态更新采用候选提交。只有 I0、D0、F13 都匹配同一个 programme/attempt/transaction/source/dispatch/AU 与 Flat7xSourceGeneration，且 F15 atomic append 返回 durable success 后，才提交 D0 overlap/history 和 next generation。F15 失败时 retire 整个 attempt，丢弃候选 history；不能只发布 I0，也不能用失败 attempt 的 history 继续下一帧。

## 接口交接

D0 producer 从当前 Flat7xPairSourceBinding 读取 4 个已投影 D0 rows，保留其 Flat7xSourceRoleMapReceipt。producer 返回 source-bound D0 frame candidate，含：

- 4,096-byte D0 frame 和 Flat7xD0FrameAssemblyReceipt；
- programme/attempt/transaction/source/dispatch/AU identity 与 Flat7xSourceGeneration；
- D0 channel-map role receipt；
- F12 IntegrityF12SourceReceipts，绑定这一个 D0 frame 的 hash、CRC 和 frame extent；
- 6 个 audio-block receipts、frame-history before/after identity；
- valid_samples、padded_samples 和 frame-level aux-data extent。

F13 仍使用 MetadataSourceProposal。producer 只接收完整 EMDF 容器和其 source receipt，不构造、不复制 JOC payload。F15 使用现有 Flat7xAuSourceReceiptInput，分别传入 F13、I0 integrity、D0 integrity receipts，并核对 IndependentI0 在前、DependentD0 在后。

## 验收和兼容

先跑输入向量，验证四声道顺序、6×256 样本、512 点窗口、U4/U7、BAP mantissa 组、C=511/512、aux-data 余量、CRC2、同源 receipt 和 reset/EOF receipt。

代码保持 D0 profile-specific。ExistingSelectedFivePointOne、所有 5.1 rate rows、旧 I0 writer 和默认 CLI profile 不变。D0 source map 仍标记 PROVISIONAL。代码改动后运行 selected_five_point_one_keeps_every_supported_rate_on_its_single_frame_route 回归。

此包授权实现临时编码策略，不授权 DEE/encoder run 或 .ec3 生成。实际编码文件仍需 Wash 对话里的 Fresh GO。




