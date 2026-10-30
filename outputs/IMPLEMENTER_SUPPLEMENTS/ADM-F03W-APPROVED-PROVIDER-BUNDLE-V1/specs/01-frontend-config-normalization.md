# F01C 前端配置归一化子集

本条款保留早期 XML→前端时钟/layout 子集核验。完整固定 profile 的权威实现合同是 F01E；它另验证 loudness、DRC、downmix、trim、XML 入口和错误提交。本文件的局部成功不能覆盖 F01E 的额外检查。

## F01C.1 输入、标签与资源引用

输入为已取得的 job XML 文本，及 RIFF 已验证的 sample_rate=48000、channels=38、非空 data_length_bytes。长度是整数，必须被114整除，duration_samples=data_length_bytes/114。

本历史子集按无 namespace 的直接子节点查找，并曾取首个 encoder。最终实现不得沿用其重复字段行为。F01E 要求根为无 namespace `job_config`，拒绝 DTD/entity、无效 XML、namespace 和重复的已识别单值路径；输入仍按 adm 优先、atmos_mezz 备用。

输入节点的file_name与storage/local/path必须存在非空text；本子集不trim这两个text。用Windows路径组合语义形成记录用reference，绝对file_name可覆盖目录。它不打开该路径、不修改output/temp路径，也不决定实际传入的PCM身份。调用者提供的实际源资源、CLI覆盖及文件身份仍按F01/I01处理。

## F01C.2 码率与固定前端结果

encoder/data_rate必须显式给出。trim两端空白后须可解释为十进制数字整数，并由整数转换成功取得kbps。当前只接受1024；其他值拒绝，不改用另一个profile或编码器。缺失、空值、负号或非数字值不接受。

当前输出 bit_rate_kbps=1024、内部bit_rate_enum=5、layout_enum=8、target_capacity=15、sample_length_enum=1、frame_samples=1536、spatial_config_mode=0、dispatch_mode=2、route_virtual_prefix_count=11。capacity不是当次active target数。frame/priming消费者见F02；mode/layout绑定见F03R，geometry layout另见F01B/F03P。

不将未消费的constructor原内存字段改名导出为配置。其余初始化或XML setter尚未绑定的字段继续列缺口，不能从上述固定输出推断完整配置已恢复。

## F01C.3 时间字段、空值与缺省

start/end/time_base/prepend_silence_duration先取直接text；缺失或空text按空字符串，然后trim。start仅接受空或first_frame_of_action；end仅接受空或end_of_file；time_base必须file_position；prepend必须0或0.0。缺失time_base/prepend拒绝，不默认补当前profile。

append_silence_duration缺失或空text缺省0.0，再trim，只接受0或0.0。仅空白text先trim得到空串，当前恢复子集拒绝它；它与无text的缺省行为不同。

input节点的offset/ffoa缺失或空text缺省auto，再trim，必须auto。仅空白text得到空串而拒绝。具体非零时间、静音插入和显式offset/ffoa不在本分支，不能调用F01G timestamp后静默扩展支持。

## F01C.4 输出、错误与限制

输出上述固定前端数值、采样率/轨数/PCM24、duration_samples、data_length_bytes、实际选取的input标签、记录用source reference，以及已trim的四项时间控制。它是纯归一化结果，没有状态推进或I/O副作用。

shape/type、PCM整帧、XML/path、码率或时间域失败按I03拒绝，不产生可用配置。native setter和错误消息/中途状态未完全观测，不宣称与原应用错误表面相同。

下游仍必须单独验证声明profile的loudness/DRC/downmix/trims。这个子集甚至可以从不含这些节点的合成XML得到前端配置；验收特意标注该结果不是可编码的完整job。见 `../test-vectors/FRONTEND-CONFIG-LOCAL-VERIFICATION.json`。
