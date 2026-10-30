# F06OC：已执行模型路线的 callback、状态和时钟

本条款连接 F06OM、F06OP、F06M 与 F06D。目标是交付已执行 Python 模型的必要行为，不要求复制 SDK 的内存布局。模型按 callback 处理规范记录，不调用 SDK queue、snapshot clone 或 frame registry。F06OB/QM/QS/QT 保留为原生内部接口的有界研究规格；本路线不能同时套用它们的事件 gate，再重复更新目标。

## F06OC.1 入口和选定配置

采样率固定 48000 Hz，callback 长度 H=1536，16 个逻辑源，8 个有序 task，每 task 两源。源索引、分类与 LFE selector 按 F06OR。几何按 F06OG 生成；distance_mode=1，low_y_scale_exponent=high_y_scale_exponent=0。每 task output filter 关闭，shared DV32 保留。

输入 packet 恰有 generation、phase、controls、pcm 四键。generation 为 U64 正整数；phase 为字符串 NORMAL 或 DRAIN，非字符串属于类型错误。controls 为16行，每行13个U32，是owned control52载体的小端 word 值。pcm 为16行，每行1536个有限 F32 数字，按逻辑源、样本排序。整数排除boolean；F32入口接受数学整数或浮点数，排除boolean和其他类型，先 R32 后检查有限及溢出。未知键、缺键、错误形状或类型拒绝。

对象行的word5/6/7/9以U32位模式重解释为F32，而不是数值转换；这些值须有限。direct行只消费word2，其他word是未消费的U32位载体，不能把其位模式当作必须有限的语义float。其他词按F06OM消费。未消费词的变化不能产生额外更新。PCM和control必须由同一meter frontend调用同时产生。generation只是错配检查，不能独自证明来源或对象身份。

真实direct control的未定义尾部不要求取得原值。frontend适配器为owned direct行提供word2；其余未消费word可规范为0。这是独立载体的明确表示，不是把未知原内存填零后声称原值已知。若增加对这些字段的消费，须补齐对应producer。

## F06OC.2 cold、reset 和拥有关系

状态恰有 source_samples、callback_index、retired、tasks。source_samples 是整数1～2^63−1；callback_index从0开始；retired是布尔，cold为false。tasks为8项，每项恰有rows和records。rows为2个 F06M gain row；每row恰有current、target、step、remaining，分别为6个finite F32、finite F32、finite F32、U32。所有F32状态入口先R32再消费。

records为null或两个完整record60 hex。hex恰为120个ASCII字符，只允许0～9/a～f/A～F；输入大小写不改变字节，输出统一小写。比较按60字节，不按字符串拼写。cold时rows全部按F06M清为正零，remaining全0，records=null；null区别于两个全零record。

每task独立拥有rows和cache。不可变record字节可以共享表示，但可变数组不得产生隐含联动。下一状态独立拥有所有可变字段；改变输入state、packet或controls不得改变输出state。输入PCM和中间metadata不作为下一callback的预测缓存。

reset(new_source_samples)创建完整cold OAR状态，callback_index回0。它不保留old rows/cache，不靠内容hash认定新旧代次相同。全pipeline reset还须由I03恢复frontend、shared DV32、meter和其他阶段；只reset OAR不能宣称全pipeline reset完成。

## F06OC.3 callback 和 EOF 账本

normal_callbacks=ceil(source_samples/H)，total_callbacks=normal_callbacks+3。当前callback_index小于normal_callbacks时phase必须是NORMAL，否则是DRAIN。generation必须等于callback_index+1。callback_index==total_callbacks后拒绝任何新callback。只有完整成功后才将callback_index加1。

NORMAL和DRAIN都向本接口提供完整16×1536 PCM和16行control。三次drain的原始meter frontend输入为零，按F06W推进；frontend/FIR/SCP仍可产生非零尾部。本接口消费它们实际产生的PCM和同代control，不能把OAR输入自行替换为零，也不能无依据重复最后一次control。

source最后一个NORMAL callback的raw输入padding由frontend按N处理。本接口仍消费完整H个已处理SCP样本，不按有效源尾长度截短PCM，不再次清零SCP尾部。OAR本接口没有额外delay/history ring；总3072+32延迟及发布截取由F06W/F06D负责，不能在这里再丢弃3072样本。

各callback先形成一个不变的同代packet，计算期间不允许同task的新packet覆写它。不同task的计算可以并行，但输出归约必须遵守 .5 的有序求和；线程完成次序不是数值顺序。

## F06OC.4 metadata、规范记录和目标

按global source i=0～15执行 F06OM.2，source_index=i、direct_count=1、frame_samples=1536。再执行 .3 生成完整owned Source30，object_source=(i!=0)。按 .4 生成canonical record60，direct_selector=4仅用于i0。

本路线的record是独立模型拥有的规范表示：每次从15个零词生成，word10/11=1、word14=0，其余字段按F06OM写入。它不是SDK持久current槽，不能声称其未消费词也等于原生padding。这里没有frame-extension/reset-map payload，结构域即是不携带这些可选控制的已执行point-model路线；不把这个域声明叫作原生对象缺席证明。

仍须执行F06OM的spread拒绝和F06OP的消费检查。active object只支持其已定义point/default-zone/no-snap条件。inactive优先返回正零是F06OP目标步骤的规则，不豁免此前Source30/record合法性或spread拒绝。不能将不支持的component、mode、width/spread或snap强行清零后继续。本条款不改变上游原始ADM的合法性，只要求上游实际SCP/control结果落入该选定路由域。

每次都从本次规范record计算16个六路targets，不用旧目标、expected值或缓存摘要预测。安装新target仅改gain-row的target，current/step/remaining保留。

## F06OC.5 task 更新、提交和求和

task t消费global源2t、2t+1。current_records是该两源的完整60字节；changed=(old records为null或任一完整字节不同)。即使targets数值相同，也不能用目标相等代替记录比较。

先为两rows安装本次targets，task输出6×1536从正零开始。按block b=0～47递增，取对应两源样本[32b,32(b+1))，调用F06M.mix32。update_flag=int(changed且b==0)，interpolation_blocks=48。其余block的update_flag=0。零PCM仍推进非跳过gain时钟，不把remaining直接清零，也不在第48块强制current=target。

每块六路输出写入该task相同样本区间。该task的48块成功后保留完整next rows和current_records。输入cache、输出cache及其他task cache之间没有可变别名。

八task输出按task索引递增、channel递增、sample递增累加，sum=R32(sum+task_output)。sum初值为正零。不改成16源一次左结合求和，也不按线程完成顺序归约。输出为外部[L,R,C,LFE,Ls,Rs]六行，每行1536个F32。之后交F06D shared DV32，再按F06W发布；本接口不输出EC3或替代JOC producer。

完整callback成功才返回normal output和完整next state。本隔离接口可在owned working copy中计算，实现不要靠复用输入对象隐含提交。已执行Python driver按task提交的失败时局部现场不作为继续运行的状态；成功值与顺序必须一致，失败统一按 .6 退役。

## F06OC.6 错误与退役

错误顺序为：完整state形状/数值检查；retired检查；已结束检查；packet四键、generation类型、phase字符串类型、代次/phase值；全部PCM shape/F32检查；全部16行controls的shape/U32与对象消费float有限性；随后才按源顺序计算metadata、Source30、record、target；最后mix和sum。晚出现的非法控制词或非有限值不得被早出现的未支持profile掩盖。检查失败不返回正常输出或next state，不修改caller输入。

错误标签：OAR_INPUT_ERROR为字段/type/shape/非有限/入口R32溢出；OAR_PIPELINE_RETIRED为已退役状态；OAR_CALLBACK_ORDER为结束后调用或generation/phase错配；OAR_PROFILE_UNSUPPORTED为Source30/record/point目标的未支持分支；OAR_NUMERIC_FAILURE为合法有限输入在mix/sum中产生非有限值或R32溢出。

这些是隔离接口标签，不认证SDK异常身份。任何失败后按I03停止本pipeline，设置retired=true或丢弃不可验证的状态。retire(valid_state)保留其他逻辑字段、返回独立owned状态并置retired=true。唯一恢复路径是全pipeline reset/rebuild；不把“调用者输入没变”解释为可以回滚全部上游状态并继续下一callback。

## F06OC.7 完整状态摘要

OAR_MODEL_CALLBACK_STATE_V1：ASCII八字节OMCLB001，接source_samples U64LE、callback_index U64LE、retired U8。

依task0～7，每task先写两gain rows。每row依次current[6]、target[6]、step[6]的F32LE，再remaining[6]的U32LE，共96字节。随后写records存在标记U8：null为0，不写cache；非null为1，接两record的完整60字节，按local源顺序。

长度为1569+120K字节，K为cache非null的task数量。cold为1569；一次完整成功后8个cache均非null，长度2529。摘要不含ABI指针、源码地址、输入PCM或前次输出；内容摘要不代替拥有关系或同代packet来源证明。

## F06OC.8 组成向量

OAR-MODEL-CALLBACK-VERIFICATION.json的cases分别从input_source_samples创建cold状态。按steps推进；RESET读取source_samples创建新cold，CALL读取input_generation/input_phase/control52_words/pcm_rule构造packet。每步输入只取显式配方，expected_state不能作为下一步种子。

pcm_rule按row r=0～15、sample s=0～1535生成，然后R32：AFFINE_DYADIC为((s mod32)−16)×(r+1)/32768+((ordinal mod3)−1)/1024；IMPULSE在s=(97r+13ordinal) mod1536时为(r+1)/32，否则正零；ZERO全正零。拒绝用例还使用CONSTANT，为给定value_f32；随后按pcm_overrides有序覆盖指定row/sample值。overrides用于验证输入错误，不参与正常运行供应。

比较完整records、Source30、targets、各task changed/update blocks、next state和完整6×1536 F32LE output SHA256。state摘要按 .7。每步采用自己的计算结果推进，并验证改变旧state或输入control不会改变新state。rejection_cases从各自input_state开始，按配方和overrides构造packet，检查expected_error及输入未修改。

研究侧已与冻结Python纯模型比较7组、51次连续callback、1次reset，输出470016词、816个gain rows及完整cache/目标/Source30一致；另有12个隔离拒绝用例。没有新读二进制、原表、私有key或真实PCM；几何按数学规则生成。该证据是合成组成范围，不是新的整首ADM冷生成或SDK内部状态比对。r2 隔离独立复审通过 7 组、51 次 callback、1 次 reset、12 个拒绝用例及 942 项检查，差异和新增必要规格问题均为 0。严格 phase 类型、全控制预校验和 direct 载体适配说明已纳入本次审查。审查限定于交付的模型消费路线，不认证 SDK ABI、冻结模型身份或原生全域行为。局部状态及 reviewed/current SHA 见 OAR-MODEL-CALLBACK-LOCAL-REVIEW-STATUS.json。

本路线明确了八项OAR连接中的模型消费接口。未消费的SDK queue/ref/opaque payload不自动成为本路线算法依赖；实际ADM→SCP/control生产、shared DV32、完整meter和全链JOC/EC3仍按各自条款验证。不能因本条款存在就标记整包就绪。仅内部研究/实现，本条局部记录不单独决定当前整包状态；当前整包 PRE_HANDOFF 为 `IMPLEMENTATION_READY=YES`，所述真实冷执行、独立实现验证和产品验收仍为下游 `NOT_RUN`。
