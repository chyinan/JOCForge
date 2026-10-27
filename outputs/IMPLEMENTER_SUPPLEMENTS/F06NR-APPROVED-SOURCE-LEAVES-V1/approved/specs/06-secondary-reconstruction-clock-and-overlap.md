# F06NR：none DRC secondary 重构连接与 overlap

状态：`SELECTED_SECONDARY_FFT_LOCAL_REVIEW_PASS`。本条定义帧连接、carry和window overlap；完整inverse256由F06NI/F06NF供应并通过局部r1复审。仅内部交付/实现，不包含研究源码或原表。

## F06NR.1 输入、配置和状态

固定48 kHz、5.1、1024 kbps、无coupling、无SPX，fullband有效谱项为217。输入为同代F07已延迟一次的六路1536 F32，外部顺序L,R,C,LFE,Ls,Rs；若按字节传递，恰为36864字节F32LE，row后sample索引。不得再运行input-history gather或借解码后的PCM代替。

flags为30个严格整数0/1，按block0..5、内部fullband lane L,C,R,Ls,Rs排列。它们是F07 `short_consumed`，历史字段名 `long_flags` 不取反；1选择short、0选择long，不使用short_new或other。

状态含五路各128个forward history、五路各128个inverse carry，以及连续U64 frame_index。冷初始化全部F32正零、frame_index=0。两组history独立拥有；inverse carry不得与forward history共用。本算子在包括前两codec priming源帧的每一源帧都执行；发布AU时抑制priming不抑制此状态推进。

frame_index必须与调用方source-frame号相同。同一个programme在一帧成功后才提交整个下一状态，frame_index加1。不以AU号代替source-frame号。输入/参数/状态不符时不返回正常secondary，按I03停用整pipeline；完整冷重建才恢复。输入类型/shape/flag/代号错误报 `SECONDARY_INPUT_ERROR`，缺少必要inverse供应报 `PARAMETER_UNAVAILABLE`，域外配置报 `UNSUPPORTED_PROFILE`，中间非有限或溢出报 `NUMERIC_FAILURE`。这些为独立接口规则，不认证原厂域外行为。

## F06NR.2 每帧计算和carrier装配

按内部lane L,C,R,Ls,Rs顺序遍历，外部槽分别为[0,2,1,4,5]；每lane内按block0..5递增。

1. 从delayed输入该外部槽取block的256样本。按本block的short_consumed及该lane持续forward history运行F08 PRIMARY long/short，获得256个未量化谱项和next forward history。这里是独立重构实例，不消费主编码器随后BAP/mantissa量化后的谱，也不共享其可变history。
2. 将谱项0..216按位保留，217..255明确写F32正零。数组仍有256项，不缩成217项后交inverse。
3. 同一个short flag选择对应inverse256算子，输出256个有限F32 transformed。其谱拆分、descriptor pre/post与clamp已由F06NI完整定义；实际FFT网络和专用参数已由F06NF供应并通过局部复审；不能用系统resampler、直接primary复制或任意DCT顺序补缺口。
4. 按.3执行overlap，使用该lane旧128 carry；保存完整next carry。
5. 每一帧都计算block0并推进两个history，但block0的overlap输出不写入本帧secondary统计carrier。block1..5各输出256样本，分别写入该外部槽carrier的block0..4。

返回前新建六路1536正零carrier，再进行上述25个256样本写入。外部LFE槽3的全部1536样本，以及其他五槽的最后256样本，明确保持正零。正零来自OWN构造规则，不是假定未初始化内存。统计F06ND的secondary仅读取fullband block0..4；block5辅助统计改读primary。

block0丢弃规则每帧都执行，不仅在冷帧一次执行。丢弃的输出不丢弃状态更新。每帧仍有30次forward/inverse/overlap。EOF只结束实际已交付source帧，不增加secondary专属zero帧、不为carry另行flush；下一programme完整清两组history和帧号。

## F06NR.3 256输出与128 carry的overlap

window为256个有限F32。所选数学路线的窗口由 `parameters/ordinary-transform-math-v1.json` 的window字段提供；位模式按U32解释成F32，不能从十进制显示近似重建。该供参用于本局部运算验收，不单独认证其与原inverse供应表的身份或整个重构。

以下每次乘、加、减分别R32，nearest-even，无FMA。Neg只翻转F32符号位；正零/负零保留。设D[256]为transformed，P[128]为旧carry，W[256]为window，out[256]新建正零。对k=0..63：

- h=W[255−2k]，l=W[2k]，s=P[k]，c=D[192+k]。写 `out[2k]=Neg(s⊗h)⊖(c⊗l)`，`out[255−2k]=(h⊗c)⊖(s⊗l)`。
- l=W[2k+1]，h=W[254−2k]，s=P[127−k]，c=D[63−k]。写 `out[2k+1]=(c⊗l)⊕(s⊗h)`，`out[254−2k]=(s⊗l)⊖(c⊗h)`。

各输出索引恰写一次。构造完整output后，next carry为D[64:192]的完整128项按位复制，不是output中间段，也不是加窗后的值。所有读取使用旧P，不能边写carry边覆盖后续读取。overlap不额外把out clamp到[−1,1]；inverse算子的前置clamp行为须在其自身条款完整定义。

## F06NR.4 局部验收与剩余依赖

`test-vectors/SECONDARY-OVERLAP-CARRIER-VERIFICATION.json` 提供六块连续input recipes和完整out/carry期望词，各实现只能用自身实际next carry继续，不能使用expected state种子。输入覆盖正零、符号零、各边界脉冲、dyadic序列和交替信号。window同样取本包数学供应文件；输入、window和carrier按F32LE计算完整SHA256。

另一个carrier配方给出五lane×六block的明确有限F32数组，独立验证25个输出块的槽/帧映射和2816个正零尾/LFE字。此配方只验证连接，不声称这些数值来自真实inverse算子。

研究側冻结overlap数学叶后比较：六连续块1536输出词与768 carry词逐项一致，source身份未变。双方运行时均消费本包生成窗口，不使用原窗口表作预测；该结果不认证原窗口身份、完整inverse或真实secondary。局部r1 fresh审查已覆盖实际FFT64/128、10组inverse和24块连续链，共760项检查零失败；真实整曲重构与音频品质仍需验收。本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
