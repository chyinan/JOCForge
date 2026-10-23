# 单元适配协议 U00

此协议连接新实现与验收 runner，不包含任何产品算法。entry_id 是独立功能入口编号；实现者按下面的粒度编写适配器，再调用自己的数值实现。每个 case 独立开始，不能把前一 case 的状态沿用过来；case 内 blocks/frames 顺序处理。

## U00.1 传输

stdin/stdout 为 UTF-8 JSON Lines。每个请求含 protocol、id、entry_id、functional_id、input 和 attachments；一行一请求。stdout 每请求返回一行 `{ "id": "相同id", "actual": { ... } }`，诊断写 stderr。响应顺序与请求顺序一致。runner 仅发送 input 和实际输入附件，绝不把 expected 发送给适配器。

JSON 中 F32 数字字段先把该十进制值按最近偶数舍入转换成 binary32，再执行对应条款；`*_u32` 是原始无符号位模式，不做数值转换。整数/布尔/null 必须保持类型。输出 hex 使用小写、无分隔符；输出 arrays 按规定顺序，bool 不能代替 0/1 整数。

输入的 file 是 unit 目录内相对名字。history 的实际输入附件名 input_file_base64，内容是 input file 的完整 bytes。history 响应 actual 必须含 output_base64、terminal_base64、delay_samples=1024 和 state_samples_per_channel=1280。runner 在收到响应后才与 expected binary 比较，不发送预期文件。

其他响应 actual 的字段集合就是该 case 的 expected schema；这些字段是本协议的输出投影，未列在投影中的内部诊断不放入 actual。expected 中的文件名只用于 runner 读取验收数据，不能当作数值结果或由适配器返回。

## U00.2 各入口

| entry_id | 操作粒度、状态和输出 |
|---|---|
| codec-budget | 固定 profile 的 F09.1；rate/blocks/sample_rate/acmod 显式输入；输出 q31、budget_words、frame_bits |
| primary-exponents | F09.2 单行 coefficient_u32；输出 clipped_u32 和 primary_exponents |
| power-exponents | F09.2 单行 power_u32/bias；输出 power_exponents |
| fullband-group-plan | F10，一通道六块；raw_row 重复 repeat_blocks 次，gains/strategies 显式；输出完整六块，wire_* 表示该块字段，effective_* 和 decoded 表示内部消费值 |
| fullband-anchor-zero | 同一 F10 六块流程，只返回 block0 的 base/groups/decoded/second_pass 投影；不是另一种规范化算法 |
| lfe-normalize | F10 单行七项，span/option/gate 显式；输出 base/groups/decoded/second_pass；不包括 LFE refresh producer |
| db-selector-from-means | 从 F09G.3 开始，跳过均方 producer；mean_six 为内部顺序六路，selector/completed_blocks 是输入状态；必须使用input.parameters中的显式合成参数；输出 selector、level_u32 和新 completed_blocks；不验证生产D07 |
| mean-square-four-lanes | F09G.2 单通道256样本，四个冷累加器；输出 mean_u32；U-MEAN-01 用原始样本验证归约 |
| gain-range-from-constant-blocks | F09G.1 六路内部顺序；每个 block_amplitudes 的每值重复 samples_per_block 次，cold=true 从六个+0峰开始，块内连续；输出 summary/code arrays、terminal peak 和 completed_blocks |
| transient-single-channel | F07 两/三节及队列；单通道冷状态，六块，每块256；recipe 明确定义第一块首样本0.5其余全部0，输出每块四个整数 flags |
| six-channel-input-history | F07.1，cold=true，输入附件为完整 frames；逐字节历史复制，返回上述 base64 输出及 terminal；内部顺序依 I07 外部存储槽 |
| programme-counts | F02.2；source_samples 为 ADM 域；dplm_callbacks 就是 meter 通路 callback 数，不是 main callback |
| dialnorm | F06.3 的 selected_lufs→5-bit 整数；不包括 meter |
| converter-fixed-cost | F11.2，bsi、six-by-five strategies、LFE/db、aux 字段显式；输出 header/fixed/aux costs；bsi 限于当前 none/mixing 组合 |
| converter-search-with-zero-bap | F11.3～4；合成 callback 对任何 candidate 返回六块、每块六路、前五路217/末路7个零BAP；实际执行搜索并记录 candidate_order；不是生产 BAP 验收 |
| emdf-single-payload | F13.1～3，一个显式 config/body 与 protection bytes 的 payload；输出 container_hex；body 不要求是生产 JOC |
| auth-bit-compaction | F14.1 的任意偶数 bytes 原语及 closed_exclusions；输出 basis_hex、kept_bits、word_count、uncommitted_bits |
| protocol-hmac-capacity-one | F14.2，显式合成 key/audio/zero-body，冷 capacity1；输出完整及截断两组 digest；不推进其他模块 |
| standard-hmac-sha256 | P04/P05 标准 HMAC，允许 RFC 向量的20-byte key，返回 digest_hex；不是生产 protocol key 验收 |
| crc16-polynomial | F14.4 任意消息 bytes 的 CRC 原语，返回2-byte crc_hex；生产 AU 另由媒体 runner 验收 |
| riff-pcm24-read | F01.2～3，riff_hex是真实合成bytes，first_sample/sample_count显式；request_metadata=null时不请求payload；返回读取投影或规范化INVALID_INPUT；合成axml/chna不验证ADM link；bool index必须拒绝 |
| metadata-fifo | F13.5，按operations顺序push/pop，初始空槽；push含tag/长度/started/phase，pop返回完整记录和buffer ID；失败时记录局部队列快照并停止该case，不验证全programme回滚 |
| id1-half-db | F13.6，measurement_u32为F32位模式，true_peak显式bool；输出整数codes |
| id1-timer-wire | F13.7～8，显式合成period_samples，cold remaining取这些periods；configurations按source调用顺序；输出每call的9项配置、remaining/cached、refreshed_field、payload_hex及terminal；不使用生产周期 |
| transform-fullband-ab | F08.1～6，每block256项U32、short_flags显式，两个128项历史冷正零，跨块连续；完整消费input.parameters的合成表，输出两路256项和各块/最终历史的原始U32 |
| transform-lfe-ab | F08.7～9，同样冷两bank历史和合成参数；无short flag；每block输出两路7项、各块/最终128项历史 |
| adm-timestamp-samples | F01G.4，timestamps按输入顺序逐项转换，输出ACCEPTED/sample或REJECTED/INVALID_TIMESTAMP；只验证time原语 |
| adm-graph-read | F01G.1～3，riff_hex为完整合成38路PCM容器，CHNA仅两轨诊断；输出引用图投影或REJECTED_MODEL_GRAPH/INVALID_ADM_GRAPH；不执行对象事件安装或全链profile选择 |
| emdf-carriage-checksum-profile | F13.9/F14K，两个显式合成bodies；mode缺省/compat/etsi按规格选择完整配置并写reserved字段，计算CRC32 primary4和CRC8 secondary1；输出配置/零槽容器/checksum/最终container；未知/null/bool mode返回REJECTED/UNSUPPORTED_EMDF_CARRIAGE_MODE；不调用私有key |
| shared-triangle-window | F04A.2，生成2N F32窗，返回整个F32LE窗SHA及显式probe_indices的U32；N=1536 |
| window-activity | F04A.2，previous_points/current_points为(index,U32)稀疏输入，其他样本正零；返回active bool、quiet int、最早witness或null及复制raw current history的SHA；不执行regular音频mute reset |
| positive-component-matrix | F04A.6，48路每路3072，未列source正零，列出的value_u32重复整row；15×48未列coef正零，destinations为完整15项映射；按正coef和F32顺序累加；返回每row SHA与sample0/中点/末项U32 |
| component-overlap-ring | F04A.7，15row冷状态；每call的mixer head/tail各1536、filtered各1536，row r取offset+r；preview不提交，最终从同一旧state提交一次；返回每call overlap/type10与terminal overlap/ring的row SHA及三点U32 |
| component-filter64 | F04A.8，15row冷64项history；block_amplitudes每值重复64；parameters全部合成且显式；输出pre/post clamp的每row SHA/三点U32及terminal source history；不调用parameter producer |
| component-parameter64 | F04P，input.block_amplitudes_u32[b][r] 是每块每 row 的原始 F32 位模式，重复 64 次形成 PCM；从同一 cold state 按 b 连续执行。parameters.coefficients_u32[j] 对应 c[j]，另外三个同名 *_u32 对应 reciprocal_pivot、target_pivot、envelope_weight；输出 records[b].peak_u32 为 next.previous_peak，scaled_peak_u32/attenuation_u32 是该块诊断值而非独立持久状态；全部参数为显式合成值 |
| receiver-ring225 | F04R.5，cold write=224/read=0；按 sample_count 生成 j/1024，再应用 points_u32 覆盖；返回输出、terminal ring 摘要和读写索引，selected_output_u32 按 probe_indices 顺序 |
| receiver-temporal | F04R.7，单 lane、输入 span、cold 双 span buffer；按 calls 连续执行，返回每次完整输出 U32、索引及最终 storage 摘要；测试包含 N<span 和索引等于 2span |
| receiver-accumulator-step | F04R.4，单 row/lane，current/target/slope/remaining 显式；每 call amplitude 重复 32 项，output 在声明的 calls 之间继续累加；参数均为合成值；这与完整 dispatch 每 callback 清 output 的范围不同 |
| receiver-transform-delay | F04R.5，6 lane×1536，按 input_recipe 生成输入，cold 32 项 delay，使用 PG01 N32；返回各 lane 输出和 terminal delay 摘要 |
| joc-quantize-values | F05Q.2，按 values_u32 顺序逐项读 F32，输出 ACCEPTED/quantized 或 REJECTED/INVALID_JOC_QUANTIZER_INPUT；不消费 fit 状态 |
| joc-representation-wire | F05Q.1～6，objects 的 points_u32/sparse_backups_u32 为原始 F32；header 显式；按下面合成 codebook recipe 写 body，返回 hex、有效 bits、header/coefficient bits 和 quantized_points |
| joc-capacity-wire | F05Q.7，在同一 wire 操作上按 capacity_bytes 和 lower_band_maps 执行全部 retry；返回完整 attempts 和 objects_after，失败边界名称仅为 COEFFICIENT_SCRATCH_LIMIT 或 FINAL_BYTE_CAPACITY |
| joc-retry-once | F05Q.7，仅执行一轮优先级最高的降级；返回 objects_after、retry_rule 和 changed bool；不写 wire，不推进任何成功 clock |
| joc-success-sequence | F05J.3，cold counter=-1/successes=0；success 按 repeat 逐次取得 sequence 并提交，trial_rejected 取得同一候选 sequence 但不提交，reset 恢复 cold；返回每操作 emitted 列表、counter、successes；这里只验收成功 clock 原语，不表示实际 programme 允许失败后续编码 |
| qmf-fft64 | F04QF，input_u32 为 128 项交错复数，mode 严格为 analysis/synthesis；使用指定参数路线及包内生成复根，输出自然频率顺序 output_u32[128]；无跨 case 状态 |
| qmf-reorder | F04Q.3/F04Q.5，input_u32[128] 按位排列及 XOR；返回 analysis_output_u32 和对该结果预格式化的 inverse_output_u32；有限 subnormal 和正负零保留原位模式 |
| qmf-synthesis-history | F04Q.6，显式合成 prototype_u32[640]、history_u32[576]，按 calls_u32 的 128 项输入逐 call 更新；返回每次 output_u32[64] 和 history_u32[576]，右侧读取旧历史快照 |
| qmf-surround24 | F04Q.4，cold 十个零 slot 和 40 项 FIR 历史；每 process 按 recipe 生成 24×128 F32，显式 mode 和 real/imag taps 各 21 项；reset 恢复两类 cold 状态；逐 operation 返回全部 output/delay/FIR 的 U32 数组，reset 无音频输出 |
| descriptor-control | F04D，polynomial_u32 按 [offset,linear,quadratic,headroom_step,result_scale] 绑定，gain_u32[128]/threshold_u32[127] 显式；cold 如 F04D.1。每 process 的 points_u32 为 [lane,sample,U32] 覆盖，未列项正零；同一 scratch 先 select 后 gain，操作间连续，reset 恢复全部 cold。返回 index/branch/peak/level、六路完整输出 SHA、sample [0,255,256,1300,1535] 的 probes 和完整 descriptor state |
| qmf-prototype-install | F04QP 的独立合成加载测试，QWIN[j]=R32((j-320)/1024)，生成 word_count 项后应用 overrides_u32；test_identity_sha256 只属于验收域，绝不覆盖生产 F04QP 固定身份。按长度→finite→identity 验证，失败返回规范化 error_class；成功返回原序/反序及两类 cold history SHA。clock_after 必须等于 clock_before；测试不代表真实生产 QWIN 已取得 |
| id2-profile-enums | F13D2.1，按 profiles 顺序转换，返回外部 enum 与内部 code 或 REJECTED/UNSUPPORTED_DRC_PROFILE；不扩展目标 programme 的 DRC 支持范围 |
| id2-profile-from-header | F13D2.1～3，type11_header_hex 为独立合成的局部 header；line/RF profile 显式；成功 configuration 顺序为 [phase_indicator,format_mask,lfe_mask,line_code,rf_code]，返回 body hex 和 valid_bits=21；拒绝为 REJECTED/INVALID_ID2_PROFILE_OR_TYPE11_HEADER；不验收完整 Type11 |
| id2-field-wire | F13D2.3，parameters 使用上述五个命名字段，只作显式整数输入序列化；返回 payload_hex、valid_bits=21、bytes=3；测试包含 bitwise AND，与逻辑 AND 不同 |
| oamd-quantize-xyz | F05O.1，逐 xyz_q15 整数三元组量化，返回 ACCEPTED/codes 或 REJECTED/INVALID_OAMD_XYZ；不消费 decode table，不代表空间控制 producer |
| oamd-escaped-integers | F05O.3，逐 values 按四组上限编码，补零到 byte 后再读回；返回 hex、valid_bits、decoded 或 REJECTED/INVALID_OAMD_ESCAPED_INTEGER |
| oamd-rowdata-wire | F05O，rows 的 kind 严格为 LFE/OBJECT；其他命名字段与条款相同，width/ancillary固定0；timing含base/position/duration/precision，trim明确不存在；返回完整 body hex/bytes/trim_present=false；大row_count只为wire诊断，不扩大programme容量支持 |
| oamd-body-read | F05O decoder，payload_hex为实际独立合成body，使用PG06网格；返回读回timing、完整rows投影和trim_present，不从encoder输入行回显。kind为LFE/OBJECT，mute/gain/spread/xyz/component/z_valid/diffuse按decode语义，省略字段使用F05O默认值 |
| oamd-trim-wire | F05T.3～4，显式 mode、group_flags[9]、groups[9][5]、row_flags[row_count]；输出仅 trim 内容的 hex/valid_bits及独立读回 decoded字段，不包含 section control或length |
| dbmd-default-trim | F05T.1～2，chunk_hex为合成 dbmd chunk内容，row_count显式；返回 ACCEPTED及完整Trim字段或 REJECTED/INVALID_OR_UNSUPPORTED_DBMD_TRIM；不从参考Type11 body读取trim |
| fit-control-records | F05H.1，xyz_u32[45]/attributes_u32[15]，构造15×14原始U32；输出 records_u32，不进行marker或OAMD decoder投影 |
| metadata-three-slot-history | F05H.4，capacity显式；cold constructor generation输出名为CONSTRUCTOR。deliver含 records_u32、markers、generation，逐操作连续；reset恢复三槽。输出各槽完整record词及generation/producer_row、delivered、control52_u32、quiet_pair_copy和initial_markers；同字节不合并generation |
| fit-marker | F05H.2～3，15target描述和current/previous状态显式，matrix_u32按active_target_indices的ordinal×48排；old/new_amplitudes各15项，每值R32后重复1536样本。由当前XYZ/attribute先构造fit，再选择marker并覆盖头两词。返回markers/records_u32/next_active以及witness、changed、distance/energy U32与energy_branch_consumed；两energy分别属于old overlap和current mixer head |
| rectangular-nnls | F03N，context_u32为15×n、response_u32[15]，cold局部状态；返回seed_u32、solution_f64_le_hex（每项8-byte小端hex）、status、passive/inactive/permutation、accepted/rejected/removals、iterations和3n上限；indices按原变量ordinal。rejected每项字段为variable/reason/passive_count/rank_increment，removals为variable/passive_position，类型、顺序和重复规则见F03N.7 |
| outer-energy-fallback | F03S.2，context/response/seed显式，分别计算branch和fallback，即使branch=active也只诊断fallback原语；返回energy/threshold/scale U32、active bool和final_u32，不表示已执行完整active路线 |
| outer-postprocess | F03S.5，coefficient_u32、floor_u32和dominance_ratio_u32显式；返回after_floor_u32/final_u32；用例参数为合成值，不覆盖生产参数选择 |
| outer-geometric-correction | F03S.3，target_xyz_u32[n][3]、source_xyz_u32[3]、scalar_flag和index_state[n]显式；返回完整left_u32/system_u32/correction_u32；不执行F03S.4能量更新 |
| outer-active-update | F03S.4，context/response/seed/correction原始U32数组显式；返回discriminant_u32、step_u32、scaled_seed_u32、output_u32，不提前floor/prune，不声称correction由完整producer计算 |
| outer-empty-matrix | F03S.1，n=0、source_call_order显式且合法；返回matrix_u32=[]、shape=[0,48]、solver_calls=0、empty_targets_skipped=true；不读取response/XYZ或运行NNLS |
| outer-iteration-limit-consumption | F03S完整单source调用，context/response/target/source XYZ显式；source0被调用、其他47列保持零，index_state清零、scalar0。返回matrix_u32、shape、nnls_status/iterations/seed_u32及energy_branch（ACTIVE_CORRECTION或LOW_ENERGY_FALLBACK）；保留ITERATION_LIMIT，不标收敛 |
| outer-rejection-probe | operation为rectangular-nnls/outer-geometric-correction/outer-active-update，arguments遵循对应接口；显式非法或数值frontier输入返回REJECTED/INVALID_OUTER_INPUT_OR_NUMERIC_FRONTIER，不传源异常文本。该入口仅验证局部拒绝，不表示programme可失败后继续 |
| candidate-selector | F03C，candidate_ids[N]、feature_u32[24][N]、positions_u32[N][3]、width_u32[N]、attributes_u32[N]、response_u32[15][N]、previous_ids[15]显式；cold调用scratch。返回status、按顺序selected_ids/initial_ordinals/attributes/count、初始affinity/complement/support原始U32及每轮gate/tie/mask/coverage/support/surviving IDs投影，字段与case schema一致 |
| selector-score-choice | F03C.5，primary_u32/secondary_u32同长，直接选择；返回chosen当前ordinal、ties、primary_max_u32/secondary_max_u32；不运行feature/coverage producer |
| selector-pairwise-affinity | F03C.2，positions/attributes/width原始U32，capacity固定15，返回原始有序pair affinity_u32；不执行complement对称调整 |
| u04-projection | F03U.4～5，source_xyz_u32[N][3]、target_xyz_u32[T][3]、target_attributes_u32[T]、source_weight_u32[N]、gain_u32[N][T]显式；输出normalized_m_u32[T][N]/projected_xyz_u32[T][3]，两层reciprocal和protected锚点按规格；gain/weights均为合成输入，不表示已执行完整solver |
| u04-admission | F03U.1，selector_status和selected_count显式；返回refinement_called、selected_count、initial_positions_preserved。仅CAPACITY_REACHED进入当前调用路径，EXHAUSTED即使count15也skip；只测caller判定，不产生membership |
| feature-fft4096 | F03F.2，3072样本正零再由points_u32[index,word]覆盖，使用固定数学FFT参数，返回自然bin0..2047交错4096词的spectrum_sha256、probe_indices指定词及word_count；不再乘窗 |
| feature-spectrum-state | F03F.3～5，parameters显式合成FFT长度/coefficient/group/fast_decay/slow_retain，单source cold vector/scalar；update用给定spectrum_u32作为假FFT输出，只测power/group/双pass和state，不测音频FFT。flags按[carry,suppress]，spectrum=null时不提供音频，carry/reset必须零FFT调用；reset恢复cold；返回vector/scalar原始词、fft_calls和branch counts |
| feature-source-gate | F03G.1，24×48未列feature正零；两flag arrays显式；返回candidate、重排feature U32、solver order、48项sum U32 |
| target-slot-install | F03G.2～4，完整合成old state/new selected；n0保留old active；返回mapping/cost/全部状态投影或REJECTED/UNSUPPORTED_TARGET_INSTALL_INPUT；不执行constructor/selector/marker |

## U00.3 Wire 与 effective 的区别

fullband-group-plan 的 reuse 块 wire_base=null、wire_gain=null、wire_groups=[]，表示字段省略。effective_base、effective_gain 和 decoded 总有值，取该区间 anchor 的规范化/合并结果。null 不是给 BAP 的增益，也不是整数0；非零策略的 wire 和 effective base/gain 一致。

当前三个 U-GROUP 的每个区间有效 gain 都为1，reuse 块也必须 effective_gain=1。LFE 没有 gain 字段。

上述F10用例的input.normalization_factor_u32是自定义合成参数，必须原样消费；它不提供D06-N生产位模式。瞬态用例的clear_threshold_u32同样是自定义阈值。所有需要parameters的入口禁止换用某个compiled默认值。

前端row摘要中的probes_u32依次为sample0、floor(sample_count/2)、末项；row_sha256为该row完整F32LE bytes，不是三个probe的摘要。所有recipe先R32输入再按条款运算，不能为匹配摘要直接返回fixture期望值。

receiver 入口的摘要字段为 sha256、sample_count、probe_u32；三个 probe 同样按首项、中点、末项排列，摘要覆盖完整 F32LE 数组。两类摘要的字段拼写不同，适配器须遵守每个 case 的 schema。

JOC 合成 codebook recipe `SYNTHETIC_FIXED_WIDTH_SYMBOL_CODES_V1`：full_coarse/sparse_coarse 各 symbol 0..95，以自身整数值作 7-bit codeword；full_fine/sparse_fine 为 symbol 0..191、8-bit；position_5 为 0..4、3-bit，position_7 为 0..6、3-bit。它们是 prefix-free 合成测试数据，不能用于真实 JOC 播放。lower_band_maps 的 JSON key 是原 band_enum 的十进制字符串，值为新 band 到旧 band 索引数组。objects_after 使用与输入相同的 U32 schema。

错误投影由entry定义，不传源异常文本：RIFF为REJECTED_MODEL_READER/INVALID_INPUT，ADM graph为REJECTED_MODEL_GRAPH/INVALID_ADM_GRAPH，timestamp逐项为REJECTED/INVALID_TIMESTAMP；FIFO错误记录error_class=FIFO_INPUT_OR_OCCUPIED并停止该case。case内允许读取错误后的局部快照仅用于证明该push拒绝未写槽，不表示programme可继续编码。其他未提供negative-case的入口，其通用错误协议/reset调用覆盖仍未就绪。

## U00.4 运行

adapter argv 文件是 JSON string array，例如 `["rtk","proxy","python","-B","新实现适配器的绝对路径"]`。该文件由实现者放在独立工作区。

从包目录运行 `rtk proxy python -B test-vectors/run-unit-tests.py --adapter-argv-file 该JSON文件 --report 单元结果.json`。runner 自身只做比较，运行输出不能证明适配器遵守文件白名单。独立实现尚未提供时，执行状态为 NOT_RUN。
