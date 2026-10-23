# F06KG：48 kHz K-filter 矩阵的数学生成

本条款提供 F06W.3 的全部64个矩阵消费值。参数来自公开二阶滤波系数及明确的固定profile归一化规则；不读取原研究矩阵。范围固定为48 kHz、modern、两个stage。其他采样率或profile返回UNSUPPORTED_PROFILE。

## F06KG.1 基础参数与来源

基础系数见parameters/meter-k-matrix-math-v1.json，按十进制字符串对应的精确有理数解释，不能先转F32或R64。stage0使用ITU-R BS.1770-4 Annex1 Table1的shelf参数，stage1使用Table2的highpass参数。固定原文链接和表号已进入参数文件；本包不复制标准全文。

stage0的b0、b1、b2统一乘gain，反馈系数不乘。gain为nearest_decimal_1e-6(10^(-(3+691/1000)/20))，ties-to-even，结果653808/1000000。691/1000对应ITU公式(2)校准offset的幅度；3是选定独立数学profile的固定归一化参数，由完整矩阵身份核验约束。不能把它说成已认证的SDK字段或原厂设计理由，也不将本条款推广为所有ITU实现的统一归一化。

该gain不属于密钥、认证材料或秘密参数。其功能是缩放stage0分子而保持反馈极点。此完整独立规则在声明范围内与选定原消费矩阵一致，按用户授权的内部数学生成路线交付；原厂设计历史未认证不影响按规则生成，原矩阵不移交。

## F06KG.2 gain的精确舍入证据

令c=653808/1000000，l=c−1/2000000，u=c+1/2000000。对正有理bound=p/q，比较p^20000×10^3691与q^20000，即比较bound^20000×10^3691与1。

l的比较结果严格小于0，u严格大于0，因此10^(-3691/20000)位于两舍入中点之间，nearest-1e-6结果唯一为c，无tie。参数文件记录两个中点、比较方向和结果。实现可用已经定义的结果c作为参数，但生成器必须保留该精确规则，不以未界定的pow近似代替证明。

## F06KG.3 8×4基向量展开

每stage使用精确有理b0、b1、b2、f1=−a1、f2=−a2。标量递推：

`y[n]=b0*x[n]+b1*x[n−1]+b2*x[n−2]+f1*y[n−1]+f2*y[n−2]`。

八个独立基向量按[x0,x1,x2,x3,x−1,x−2,y−1,y−2]排序。对每个基向量，只将对应变量设1，其余七个为0；依n=0、1、2、3递增，以此递推计算y0～y3。已经计算的yn参与下一n，初始y−1/y−2只由该基向量指定。没有其他初始变量，也不调用滤波历史或native provider。

M[basis][lane]等于该基向量产生的y[lane]。中间加减乘全部精确有理；不融合为F32计算，不先量化基础系数。每个最终M项先量化为nearest decimal 1e−8，ties-to-even，再按N00的精确R32 nearest-even转F32。

decimal量化可写为m=RNE_integer(M×10^8)，q=m/10^8；m的符号完整保留。随后对q精确R32。数学零产生正零。四个cold-state词另附正零作为row8。因此完整结果为stage0/1、每stage9行、每行4个F32；按stage、row、lane序小端存储，共288字节。

## F06KG.4 供表与消费连接

生成结果直接提供F06W.3的stages[2][9][4]；lower/upper/noise使用已定义基础参数，不能再乘gain或补其他scale。F06W已包含输入1/8、输出1/32、逐lane累加、noise和state clamp；这些消费运算不因供表生成而改变。

每路state从两stage row8分别复制，六路独立。reset按F06W恢复完整cold。本参数算子没有跨callback可变状态；cold state包含在接口中，但不能把参数对象当作可变history。参数不全或类型/形状/来源版本不符合声明时返回PARAMETER_UNAVAILABLE；非声明profile返回UNSUPPORTED_PROFILE，不能改用测试矩阵或不同频率。

## F06KG.5 验收及限制

参数文件的generated_f32_words为生成器输出的72个U32位模式，包含64矩阵词与8个cold词；不是原编译表搬运。用 .1～3自行生成并比较全部词及完整SHA256：08dc825e016f703df13b023bf29486996cba7c1eac2ebc1c7741da54ef98fb1a。SHA仅用于身份核验，不作为运行输入。

METER-K-MATRIX-GENERATION-VERIFICATION.json先生成完整表，从initial_state_words的两stage各四词开始，按continuous_cases递增推进。每case的input.samples和sample_rule产生有限F32：IMPULSE的i0=1、其他正零；MOD37_101_OVER64为R32(((37i)mod101−50)/64)；ALTERNATING3为奇i+3/偶i−3；ZERO全正零；HALF为R32(1/2)；MOD71_193_OVER32为R32(((71i)mod193−96)/32)。

按F06W.3生成output及完整next state，再用F06W.4求power。比较输出F32LE SHA、8个state词和power词；下一块只用自己的next state，不从expected恢复。各case长度均满足8倍数及≤1024边界。该组包含连续impulse、有理调制、±3 stress、后续零输入与DC。

研究与根侧复核已通过64/64矩阵词、完整72词/288字节身份，以及6块4160输出词、48state词、6power词。原研究矩阵仅用于生成后的比较，未输入生成器；没有新读DLL、私有key或真实PCM。r2 隔离独立审查通过完整 72 词、288 字节，以及 6 块的 4160 输出词摘要、48 个 state 词和 6 个 power 词。基础参数中供表状态与 provider 不一致的问题已关闭，新增问题为 0。审查仅涵盖 F06KG 与 F06W.3/.4；公开来源、原编译表身份及原厂归一化设计理由不由本次审查认证。局部状态及 reviewed/current SHA 见 METER-K-MATRIX-LOCAL-REVIEW-STATUS.json。

本供参替换在声明范围没有数值近似，不等于完整meter或新ADM整链已验收。true-peak、frontend、getter/DRC及其他模块继续各自闭合。仅内部研究/实现，本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
