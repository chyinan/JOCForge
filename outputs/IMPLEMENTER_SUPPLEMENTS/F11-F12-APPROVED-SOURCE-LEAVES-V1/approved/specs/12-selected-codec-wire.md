# F12S selected 5.1 E-AC-3 编码与 wire

`parameters/codec-wire-selected-v1.json` 供应所选正常协议与算法常数。

本模块只支持 48 kHz、1024 kbps、5.1、六块、一个 independent substream。`acmod=7`，LFE 开启。全带宽为 5 路，每路 217 个谱 bin；LFE 为 7 个谱 bin。coupling、SPX、delta bit allocation 和 AHT 必须关闭。

输入音频、变换、指数、BAP、BSI、EMDF 和发布时钟都必须来自本包中的生产规则。实现不能读取歌曲专用 plan、参考 EC3、缓存 BAP 或参考字段来预测新输入。

## F12S.1 输入和输出

每个 source frame 输入以下值：

- 六路有限 F32 音频，每路 1536 项。外部顺序为 `[L,R,C,LFE,Ls,Rs]`。
- 六块五路 short 标志。内部通道顺序为 `[L,C,R,Ls,Rs]`。
- F09/F10 产生的六块指数计划。每块含五路全带宽计划和一路 LFE 计划。
- 六个 db-per-bit code、五个 converter frame code 和一个 converter SNR offset。
- F06 产生的同 source BSI。line DRC 是六块独立的可选 U8。
- F13 产生的可选 EMDF payload。当前 payload 固定放在 block 2，长度为 0～511 bytes。

成功输出恰为 4096 bytes。字段按最高位优先写入；多字节 CRC 按大端写入。失败不输出 AU。

## F12S.2 staged 状态

持久状态包含六路变换 history 和五路 `previous_short`。冷启动时全部为正零或 false。keyless 路线没有 tail RNG 状态。

每个 source 开始时克隆持久状态。六块变换、dither 计划和 trial 共用该副本。trial 只能读取固定谱、指数、BSI 和 payload，不得重新推进变换或 short history。

至少一个 candidate 成功时，提交 staged 变换状态和本帧结束后的 `previous_short`。所有 candidate 失败时，丢弃 staged 状态并按 I03 终止 programme。不能只重试同一个 source。

## F12S.3 dither 语义

对 block `b=0..5`、全带宽 lane `c=0..4`：

```text
dither[b,c] = NOT short[b,c] AND NOT previous_short[c]
previous_short[c] = short[b,c]
```

更新按 block、lane 递增顺序执行。当前 layout selector 为 7，coupling 全 false，因此不再屏蔽 lane。若 30 个 dither 值全 true，则省略 dither syntax；省略表示 30 个值全 true。否则写 30 个显式 bit。

BAP=0 不写 mantissa code。dither flag 告诉解码器是否为零 BAP bin 产生替代噪声；编码端不为这些 bin 写随机 mantissa。

`short_syntax` 当且仅当本source frame的30个fullband short标志中至少一项为true；存在时每块按内部lane `[L,C,R,Ls,Rs]` 写5个bit，合计30bit。省略时解码语义为30项全部false（全部long），不能把省略解释为沿用前帧或全部true。该存在判定只看当前30项；`previous_short` 仅参与dither，不参与short_syntax存在位。

## F12S.4 主 BAP

candidate `q` 为 U10。`coarse=q>>4`，`fine=q&15`。每块、每通道独立调用 F12A：

- decay code 为 `[2,1,1]`；
- db-per-bit code 取该块 F09G 输出；
- floor code 为 7；
- fast gain code 为 1；
- start bin 为 0；
- end bin 为全带宽 217 或 LFE 7；
- coupling、delta 和 AHT 均为 false。

调用者的 `all_active_fine_zero` 必须按同一 trial 的 36 个活动元素计算。只有 `coarse=0` 且所有活动元素的 fine 都为 0 时，才允许全零 bypass。当前主搜索从 16 开始，不走该分支。

## F12S.5 mantissa 量化

进入量化前，FBW PRIMARY与LFE MAIN都必须采用F09LB提交的clipped staged coefficient row；不得重新读取未裁剪transform row。输入域、strict ±1边界和失败零提交规则见 `specs/09-lfe-gate-mantissa-boundaries.md`。

对每个谱系数 `coefficient`，先在 F64 中乘 `2^decoded_exponent`，再转为 F32 得 `x`。后续每个 `+`、`-` 和 `×` 都按 N00 单步 R32。整数转换向零截断，除非条款明确写 RNE。

- BAP 0：code 为 0，不写位。
- BAP 1、2、4：`code=clamp(trunc((x+1)×scale),0,levels-1)`。`(scale,levels)` 分别为 `(1.5,3)`、`(2.5,5)` 和 `(5.5,11)`。
- BAP 3：`raw=trunc((x+1)×3.5) bitand 7`。`raw=7` 时 trial 失败；否则 code 为 raw，宽 3 位。
- BAP 5：`raw=trunc((x+1)×7.5) bitand 15`。`raw=15` 时 trial 失败；否则 code 为 raw，宽 4 位。
- BAP 6～15：宽度 `w` 依次为 `[5,6,7,8,9,10,11,12,14,16]`。计算 `offset=R32(2^-w)`、`summed=R32(x+offset)`、`scaled=R32(summed×65536)`、`shifted=R32(scaled-0.5)`、`r=RNE_integer(shifted)`、`fixed=clamp(r arithmetic_shift_right 1,-32768,32767)`。`w<16` 时取 fixed 二补码低 16 位中的最高 w 位；`w=16` 时取低 16 位。

输入非有限、BAP 不在 0～15、保留 sentinel 或不能表示的中间结果使当前 trial 失败。

## F12S.6 分组和写入

每个 block 先按 `[L,C,R,Ls,Rs,LFE]`，再按各通道 bin 递增顺序，形成 mantissa item 序列。每个 block 重新建立三个独立分组器；分组不跨 block。

- BAP 1：按该 BAP 在 item 序列中的出现顺序每 3 个组成一组。缺项填 code 1。以 radix 3 累积，写 5 位。
- BAP 2：每 3 个一组，缺项填 code 2。以 radix 5 累积，写 7 位。
- BAP 4：每 2 个一组，缺项填 code 5。以 radix 11 累积，写 7 位。

分组字段写在该组第一个 item 的位置。该组其余 item 不写位。其他非零 BAP 在自身 item 位置写 F12S.5 的 code 和对应宽度。

## F12S.7 selected header 和 block 顺序

帧起始固定字段按顺序写：

| 字段 | 位宽 | 值 |
|---|---:|---:|
| syncword | 16 | `0x0B77` |
| strmtyp | 2 | 0 |
| substreamid | 3 | 0 |
| frmsiz | 11 | 2047 |
| fscod | 2 | 0 |
| numblkscod | 2 | 3 |
| acmod | 3 | 7 |
| lfeon | 1 | 1 |
| bsid | 5 | 16 |

随后写 dialnorm 5 位、`compre` 1 位和条件存在的 `compr` 8 位。再写 `mixmdate` 1 位。存在时依次写 preferred downmix mode 2 位、4 个 downmix level 各 3 位、`lfemixlevcode=0`、`pgmscle=0`、`extpgmscle=0`、`mixdef=0` 2 位和 `frmmixcfginfoe=0`。

再写 `infomdate` 1 位。存在时依次写 bitstream mode 3 位、copyright 1 位、original 1 位、surround EX mode 2 位、`audprodie=0` 和 source sample rate code 1 位。

若有 JOC payload，写 additional-BSI presence 1、length code 1 的 6 位、identifier 1 的 8 位和 complexity 8 位。complexity 必须按 F13JO 从同一 source call 已完整验证的 EMDF 重新派生；selected profile 要求 ID14 object_count15 与 ID11 slots16，输出 `object_count+1=16`。不得从参考 header、歌曲缓存或上一 source 沿用。无 JOC payload时只写 presence 0。

随后写 selected audio-frame 控制前缀：exponent format 1 位、AHT flag 0、SNR offset strategy 2 位值 0、transient pre-noise flag 0，以及 7-bit syntax mask：

```text
mask = 64×short_syntax + 32×dither_syntax + 16×BA_syntax
     + 8×fast_gain_syntax + 2×EMDF_skip_syntax
```

其他两位为 0。当前 fast gain syntax 恒为 1；EMDF payload 存在时 skip syntax 为 1。

每块的 BA tuple 为 `[2,1,1,db_per_bit_code,7]`。若 block 0 的 tuple 不是隐式默认值 `[2,1,1,2,7]`，或任意相邻块 tuple 不同，则 `BA_syntax=1`；否则为 0。`BA_syntax=0` 时，六块都使用该隐式默认值。fast gain code 为 1，而隐式默认值为 4，所以当前路线的 `fast_gain_syntax` 恒为 1，并在每个 block 写一次更新。

接着写六个coupling控制bit，全部为0。若主exponent format为0，写五个5-bit **主** frame code；若为1，写6×5个2-bit全带宽block strategy。当前主producer先匹配F09的32个合法六块pattern，无法匹配时才使用format1。主frame code与下一段converter frame code由不同计划分别产生，不能复用。

再写六个 1-bit LFE strategy、五个 5-bit converter frame code、主 coarse 6 位、主 fine 4 位和 `block_start_info=0`。

每个 block 按以下顺序写：

1. short syntax 存在时，写五个 short bit。
2. dither syntax 存在时，写五个 dither bit。
3. 写 line DRC presence；存在时再写 8-bit code。
4. 写 SPX presence 0。
5. 对五路中 strategy 非零的通道，先按 lane 顺序写全部 6-bit bandwidth code 48。
6. 再按 lane 顺序写全部刷新通道的指数 body：base 4 位、每组 7 位、gain range 2 位。
7. LFE strategy 为 1 时写 base 4 位和两个 7-bit group；LFE 没有 gain range。
8. BA syntax 存在时写 update bit。首块恒更新；后续 tuple 与前块不同时更新。更新时写 `[slow_decay,fast_decay,slow_gain,db_per_bit,floor]`，位宽为 `[2,2,2,2,3]`。
9. 每块写 fast gain update 1，再对六路各写 fast gain code 1 的 3 位。
10. block 0 写 converter SNR presence 1 和 10-bit offset；其余块写 presence 0。
11. EMDF skip syntax 存在时写 skip presence。只有 carrier block 2 为 1；随后写 9-bit byte length和 payload bytes。其他块写 0。
12. 按 F12S.6 写本块 mantissa。

line DRC absence不写 code。策略 0 的全带宽和 LFE 均继承同帧最近 anchor 的完整 decoded exponent；全带宽同时继承 gain range。它们不写 bandwidth、指数 body 或 gain range。

## F12S.8 主搜索

先测试 `q=16`。然后设置 `low=16`、`high=784`、`trial_count=0`。只要 `high-low>1` 且 `trial_count<10`：

1. `q=floor((low+high)/2)`，`trial_count+=1`。
2. 用相同 staged 谱、指数、dither、BSI 和 payload 构造完整帧。
3. 成功时保存该 candidate 为 best，并令 `low=q`。
4. 失败时令 `high=q`。

初始 16 的测试不计入 `trial_count`。成功要求 body 在尾部前不超过 `32768-34` bits，并且 BAP、量化、分组和所有字段均成功。最终选择最后一个成功 candidate。没有成功 candidate 时 source 失败，不提交状态。

## F12S.9 keyless tail 和 CRC

完成 body 后填 0，直到还剩 34 位。依次写 16-bit `tail_word=0`、`auxdatae=0`、`encinfo=0`。最后在 bytes `[2,4094)` 上计算 CRC-16，初值 0，多项式 `0x8005`，每字节从最高位到最低位，不反射、无 final XOR。CRC 以大端写入最后 2 bytes。完整帧 bytes `[2,4096)` 的同一 CRC residual 必须为 0。

私有 HMAC key 不参与本模块。keyless checksum profile 由 F14K 处理其公开字段。它不能改变音频 body、指数、BAP、mantissa、EMDF 内容或 source/AU 时钟。

## F12S.10 source、AU 和 EOF

source 从 0 连续递增。每个成功 source 都提交 F12S.2 状态。source 0 和 1 不发布；source `s>=2` 发布为 AU `s-2`。当前完整输入为 4249 source 和 4247 AU。

codec 不增加 EOF flush。前端已把最后 486 个输入样本补零，并完成其 4 次 cached flush。codec 只消费已经形成的 4249 个完整 1536-sample source frame。

## F12S.11 EMDF 模式

`deployed_compat` 为默认，`etsi_emdf` 为可选。两种模式都先由 F13 生成并验证完整 EMDF payload，再交给本模块。模式选择不得关闭 OAMD/JOC body，也不得改写音频算法。

本模块只观察 payload bytes、长度、carrier block 和 JOC complexity。两种模式在这里使用同一 skip 字段规则。`codecdatae` 及 payload 内部差异属于 F13，不在 F12 重新解释。

## F12S.12 错误

入口尺寸、类型、范围、状态代次或 source index 错误返回 `CODEC_INPUT_ERROR`。缺 F12A 表或其他必要 provider 返回 `PARAMETER_UNAVAILABLE`。coupling、SPX、delta、AHT、非 48 kHz、非 5.1 或非 4096-byte profile 返回 `UNSUPPORTED_PROFILE`。

trial 内的容量、BAP、量化 sentinel、分组或字段构造失败表示该 candidate 不可行。trial 外的变换、计划、BSI、EMDF 或状态错误是 programme 致命错误。没有 best 也属于致命 source 失败。所有致命错误按 I03 fail-stop；重新开始时完整冷 reset。

## F12S.13 验收

`test-vectors/CODEC-WIRE-SELECTED-INPUT-VECTORS.json` 提供208个量化案例、1个跨类别分组案例、1个CRC residual案例和3个搜索案例。208个量化案例已与冻结accepted quantizer直接执行逐项比较。

`test-vectors/CODEC-WIRE-COMPLETE-FRAME-INPUT-VECTORS.json` 提供4个完整input-only构造案例。每个输入只给谱配方、指数计划、BSI、short/dither初态、db/converter输出和F13 payload；expected 4096-byte frame、bit offset、search轨迹、state与CRC均在构造完成后记录。四例都让主frame code `[0,0,0,0,0]` 与converter code `[16,17,18,19,20]` 不同，能发现两组字段被混用。

案例覆盖short syntax省略1例、mixed-short显式语法3例、实际keyless JOC carrier 1例，以及deployed_compat/etsi_emdf成对合成carrier。模式对比较删除每块skip presence、carrier length和payload bytes后，对剩余padding前语义字段按原顺序拼接；哈希前加8-byte大端bit长度。两种模式的该语义串必须同长同SHA，而完整frame SHA必须不同。绝对skip payload范围和每个字段offset分别交付，不能用旧模式的offset解释新模式。

完整向量还记录每帧失败trial数量、失败trial提交数0、成功source提交数1和next_previous_short。另有512-byte payload入口拒绝例：trial数0、提交数0、previous_short保持输入值。reset把previous_short恢复五个false。结果见 `test-vectors/CODEC-WIRE-COMPLETE-FRAME-VERIFICATION.json`。

`parameters/codec-wire-cross-module-providers-v1.json` 绑定本fixture及生产入口所需的F06、F07、F08、F09、F10和F13 clean规格、授权参数及已有输入向量。审查冻结必须同时包含该清单中的全部文件；只传清单SHA不能代替文件本体。

r2b package-only独立审查已登记为 `test-vectors/CODEC-WIRE-SELECTED-LOCAL-REVIEW-STATUS.json`：冻结清单38/38、cross-module provider身份25/25、208/208量化、12→7→51-bit分组、3个抽象搜索、4/4个完整4096-byte frame及CRC residual、两个EMDF模式容器与32487-bit语义音频串、25个失败trial零提交、4个成功source各一次提交、512-byte payload trial前拒绝均通过。该记录只把 supplied-fixture 的F12边界判为YES；F06生产链、F08新参数路线音频/render、F13实际payload producer以及4249-source→4247-AU fresh replay仍是外部blocker，本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。

整体验收还必须从实际输入重新计算 4249 source，并检查 4247 个 AU 的解码、CRC、JOC 识别和空间渲染。历史 Bit-Exact 整流只作回归基线；keyless 实现不要求与含私有认证值的整流逐字节相同。
