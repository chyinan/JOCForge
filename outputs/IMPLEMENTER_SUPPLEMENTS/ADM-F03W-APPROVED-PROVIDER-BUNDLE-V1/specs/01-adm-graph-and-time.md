# F01G ADM引用图与样本时间

本文件定义 CHNA/AXML 读取和选定单 block、零起点 Cartesian 对象分支。DBMD、bed 的 `channel_format_id` 身份安装、完整事件合并、callback hold 和 reset 见 F01E。任意多事件或非零起点 ADM 明确拒绝，不声称通用 ADM 兼容。

## F01G.1 CHNA

chunk至少4 bytes。前两U16LE分别是track_count与uid_count，实际总长必须等于4+40×uid_count。每40-byte条目依次为track_index U16LE、12-byte audioTrackUID、14-byte trackFormat引用、11-byte packFormat引用、1-byte reserved。三个字符串以首个NUL结束，剩余bytes不解释；NUL前字符必须ASCII。不清除字符串首尾空白。

按条目物理顺序输出，track_index和UID分别不得重复。读取器保留reserved，不要求它为0。这个阶段不强制uid_count=track_count，也不强制track_index连续；后续图输出完整性诊断。实际被验证profile仍要求38路合法安装，不能以容器读取成功接受任意轨数。

## F01G.2 AXML索引与引用

必须取得唯一axml/chna chunks。XML原始bytes中，不区分大小写出现`<!DOCTYPE`或`<!ENTITY`即拒绝。XML必须结构完整，根local name为ebuCoreMain。元素namespace固定取根namespace；无namespace时为空。只有同namespace节点参加后续查找。

以UID、audioTrackFormatID、audioStreamFormatID、audioChannelFormatID、audioPackFormatID分别索引对应节点，缺/空ID不进索引。重复ID采用文档中最后节点；接受模型没有额外重复ID拒绝。原生对重复ID的行为未证明。对象按文档顺序读取其直接audioTrackUIDRef子节点，text两端trim；同一UID出现多个引用时均计入关联。

对每个CHNA条目，依次解析UID→trackFormat→streamFormat→channelFormat，并取得packFormat。UID、track和pack必须存在；UID的直接trackFormat/packFormat引用必须分别与CHNA一致；track的stream引用、stream的channel引用必须有目标节点。字符串引用取第一个对应直接子节点的trim text。

UID存在sampleRate/bitDepth属性时转换为整数，并分别要求等于PCM rate和24；缺省保留null。空属性不能当作缺省数值。该UID关联的audioObject必须恰好一个；零个或多个失败。该对象可关联多个不同UID，因此对象数不必等于轨数。

输出轨道行保持CHNA顺序，物理零基track slot=track_index-1；保留引用ID、track/channel/pack名称及typeDefinition、对象id/name/start/duration。

## F01G.3 Block字段与诊断

按channel节点的直接audioBlockFormat顺序读取。保留block ID、rtime、duration；递归position子节点的coordinate转大写，同名coordinate采用最后出现值；缺coordinate使用空名。position text为空时按0，其他text转换为binary64数值。width/depth/height/diffuse各取第一个直接子节点，缺节点为null，存在但text为空按0。cartesian和jumpPosition取第一个直接子节点的trim text。speakerLabel按递归文档顺序保留trim text。

该读取层不对所有数值检查有限性或坐标域；当前消费分支须再按F01G.5校验，不能把含NaN/Infinity的图当作可编码profile。未知字段保留在原XML中但不参与本读取投影；它们的原生语义尚未核验。

输出programme id/name/start/end和content id/name列表及节点计数。track_index_domain为映射中轨号排序结果；complete表示它恰好等于1～CHNA track_count。bed/object数量按pack typeDefinition分别等于DirectSpeakers/Objects统计；LFE诊断按channel name等于RoomCentricLFE收集轨号。该名字判断只是当前读取诊断，不替代bed路由的完整producer。

## F01G.4 timestamp到sample

sample_rate为正整数，bool不允许。输入time string不先trim，语法为h:mm:ss或h:mm:ss.f，其中h为一位或更多十进制数字，mm/ss恰好两位且小于60，fraction长度1～6。小数补右侧0得到microseconds整数；超过6位、空fraction、负时间、超范围分秒拒绝。

每步使用binary64最近偶数舍入，禁止改成纯有理数：a=R64(microseconds/1000000)，b=R64(a+seconds)，c=R64(minutes×60+hours×3600)，d=R64(c+b)，e=R64(d×sample_rate)，f=R64(e+0.5)。要求f有限且0<=f<2^63，然后向零截断为signed64 sample。

各整数先精确求和/乘法，再在指定边界转binary64。该规则解释当前XML的微秒时值与PCM N之间的量化差；时间转换本身不裁剪PCM，也不建立fit/consumer时钟。

## F01G.5 当前对象事件的语义字段

对象track须pack type=Objects、恰好一个block、cartesian文本为1、jumpPosition文本为1；block rtime和关联object start均为0秒，interpolationLength表示的秒数为0。非零起点、多block、非零interpolation不支持。block duration按F01G.4转换为正sample_count，object duration必须转换成相同sample_count；二者的十进制字符串不必完全相同。

object ID必须匹配AO_加恰好4个十六进制字符。解析值必须严格大于0x1000，semantic key=解析值-0x1001，以U32表达；不是直接把文本ID或轨号当key。其他ID形式不支持。

坐标来自binary64解析值，必须有限。每个存在坐标先处理：恰好等于sentinel -1000时在输入夹取阶段保持，否则限制到[-1,1]。再对处理后X、Y、Z分别进行：X→R32(R64(R64(X×0.5)+0.5))；Y→R32(R64(0.5-R64(Y×0.5)))；Z→R32(Z)。因此 raw X/Y=-1000 在 event 中变为 -499.5/500.5；“保持 sentinel”不跳过轴变换。每坐标附presence，不存在时默认XYZ=[0.5,0.5,0]并保留该坐标presence=false，不能填入歌曲特定坐标。event 到 control 的后续夹取见 F01E.8。

存在的width/depth/height必须彼此相等；选其值先R32。等于extent sentinel -1000时保持，否则限制到[0,1]；附extent presence。diffuse存在时转换为int(value!=0)，附presence。零interpolation分支写interpolation_samples=0并附presence。默认、DBMD 合并、物理轨道安装、sampler、control52 和失败提交规则见 F01E。

## F01G.6 验收与错误

CHNA正例、重复UID/track、非ASCII、缺失link、namespace和时间舍入单独验收。合成图可使用少量关联来验证读取原语；不代表已支持少轨ADM整链。拒绝时不输出可用语义事件，programme按I03终止。复杂图、entity、重复ID和极大时间的原生错误状态仍NOT_EVALUATED。
