# F04G 参数供应与调用连接补充草稿

本文补充既有 F04G.1～9 的完整运算网络；selected 24个非秘密功能值已由F04GA供应并通过局部复审。适用域是 11 点 prefix、uniform_width_mode=0、component_enum=0、enabled=1、XYZ∈[0,1]、raw_width∈(0,1]。它与 F03P 的 15 点 feature response 是两条不同消费者。

## 参数供应接口

一份供应材料包含以下四项，所有值以精确 8 位 U32 十六进制位模式承载，按 F32LE 解释，禁止以较短小数替换。文件同时给出版本、消费域、逐组字节散列和授权范围；不含研究源码、提取工具、二进制文件或厂商认证材料。

| 字段 | 类型与数量 | 消费者与必要顺序 |
|---|---|---|
| width_pairs | 5×2 F32，40 bytes | F04G.2；每对 input_width,mapped_width，原顺序 |
| kernel_exponent | 2 F32，8 bytes | F04G.5；先乘第 0 项再乘第 1 项，不能只供应相乘结果 |
| power_cubics | 7 F32，28 bytes | F04G.4；log(a,b,c,d) 后 exp(u,v,w)，不能替换为一般 pow |
| power_and_z | 5 F32，20 bytes | F04G.5；low,breakpoint,intercept,slope,z_angle_scale |

几何依赖为独立 points[11][3] provider。本任务只定义其输入形状与 F04G.3 分组条件，几何数据由 F01B 供应，不能从 width 表或歌曲 metadata 推导。四项共 24 个 F32、96 bytes；不是四个原始表文件。

供应缺失返回 PARAMETER_UNAVAILABLE。结构不符合数量、非有限项、width input knots 非严格递增、未覆盖 [0,1]、或不满足 F04G 运算约束时返回 PARAMETER_DOMAIN。不能用零值或候选拟合系数继续编码。验证成功后允许缓存由完整参数身份、几何身份和 cell 索引构造的纯函数结果；不允许以歌曲 ID 或参考响应缓存代替算法。

## 调用视图与时间

输入字段使用 F04G.9 的独立 32-byte 控制承载协议。raw_width 是量化前原 F32，同时供 F04G.7 的 sin² 与 front compensation；spread 内部映射消费的是 R32(q_width/32768)。raw_width、量化 width、mapped_width、cell width 四者必须分别保存，不能互换。位置同样先量化再用于 runtime descriptor。

控制52-byte语义视图中，XYZ 位于20/24/28，uniform_width_mode位于32，width位于36，extra位于40，component_enum位于44，enabled位于48；本模块读控制[20:52]的完整32-byte tail。源视图在此前插入4-byte前缀，因此对应源字段位于24/28/32、36、40、44、48、52。本模块不依赖未定义的前导hold字，不以源ID填充它。13-word控制承载中的XYZ索引5/6/7、width索引9与这些字段对应。

F06OM 对同一raw width生成metadata量化宽度时使用其独立31-grid规则；那份编码后宽度不能反向替代本模块的raw width或Q15宽度。本补充只证明字段连接，metadata的量化边界、状态、源generation与提交顺序仍按F06OM/F06OC/F06OF执行。

控制采样和历史队列先产生本次有效控制，再调用 prefix producer。每次有效调用从输入和 provider 重算完整 11 项 gain；本函数没有跨 callback 的数值 state。底层 gain ramp、历史控制和按源累计由各自规格持有，不属于本函数的冷初始状态。cell cache 为空或已填充不能改变任何输出。

prefix caller 仅对 object 且 raw_width>0 且该次非 quiet 的控制形状调用。raw_width<=0 或 quiet 时没有本次 prefix gain 写入。成员求和的逻辑无贡献可表示为空集合，但不能伪造一份全零 gain vector 并声称原 producer 已写入，也不能把没有调用解释为 reset/ramp 清零。bed 路由与 object point 路由独立定义。

F03P 的 feature caller在音频路由后构造独立视图 feature_width=正零，保留原音频 width。此时 point response 只读取 feature_width；不能把这个清零 writer 施加到 prefix 的 raw_width，或以其 zero-spread 验收覆盖 prefix 的 spread 算法。

## 边界与提交

输入协议解码按 F04G.9；域检查发生在 gain 或 cell 输出提交之前。正 raw_width 可能量化到 q_width=0；该情况不自动转 point 路线。mapped_width<R32(0.000025) 时完整调用失败，即使单独 kernel 原语规定输出零。失败不产生可提交完整 11 项 gain；整管线按 I03 停止并重新冷初始化，不能继续沿用一份伪造的成功响应。

width mapping 的 exact endpoint comparisons 与每步 R32 保持 F04G.2 定义。生产域是否允许负 power 必须由具体 provider 与输入域验收确定；现有参数化算法允许有限非零负 power，不能据此声称声明生产域真的会达到负值。

局部参数检查、纯函数重算及冻结模型对比不等于真实 ADM PCM、完整空间品质或整包 readiness。独立数学候选只有在声明域建立验证后可采用；用原值编码成分数只是表示变化，不构成独立来源或自动交付授权。
