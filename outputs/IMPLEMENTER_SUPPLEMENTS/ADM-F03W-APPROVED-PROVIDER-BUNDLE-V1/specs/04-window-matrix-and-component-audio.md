# F04A 双callback窗口、矩阵与component音频

本文件补齐选定路线的窗口、activity、previous/current、矩阵及component overlap/ring/64样本滤波网络。输入路由、prefix gain producer、四路扩散滤波、参数producer与receiver另行定义；缺失部分仍阻塞全链。所有非秘密必要参数仍须洗白，不能用缺表当作略去该功能的理由。

## F04A.1 数据和冷状态

每callback输入38个1536样本的finite F32物理轨道。语义control行按物理轨道顺序覆盖0～37，包含mute整数0/1、bed/object类型、direct LFE标记、spread F32、diffuse状态。这里只支持恰好一个direct LFE；从regular路径移除它，剩余37行按原轨道顺序编号tail0～36。其轨号由ADM routing提供，不写死歌曲轨4。

状态分开拥有：37×1536 raw regular history；38×1536 previous PCM；48×1536 previous current rows；按tail ID索引的四路扩散filter state及previous outputs；15×1536 component overlap；15×1472 component source ring；15×64 scalar filter history；共享component参数state。前六种音频状态冷初始化全positive zero，filter map为空。parameter state的完整构造器另见对应参数条款，不能用这些零数组代替它。

接口输入/输出数组按各自语义顺序，不共用普通codec的1280/128项history。所有尺寸、finite性及控制域在计算前检查；错误后I03 fail-stop，不能从半更新callback续用。

## F04A.2 三角窗及activity

令N=1536。共享只读窗W长度2N：W[i]=R32(R32(i)/R32(N))，W[N+i]=R32(R32(N-1-i)/R32(N))，i=0～N-1。首次构造后跨callback复用；不是未交付静态表。两端为正零，两半最高值为R32((N-1)/N)。

activity对同tail的旧raw H与当次PCM X，按i递增计算a=H[i]⊗W[i]、b=X[i]⊗W[N+i]。存在abs(a)>2^-23或abs(b)>2^-23时记录最早witness i并停止扫描；相等不触发。无witness为quiet=1；有witness且mute=0为quiet=0；有witness且mute=1也为quiet=1。history_after始终先复制当次raw X，包括tiny值和mute样本，不保存窗乘积。

仅在mute!=0且确有witness时，regular音频的当次窗及previous音频half清为正零；未达阈值的tiny输入不因quiet而清零。activity history仍保存raw X。全局其他mute/flag不替代每行mute。

## F04A.3 Spread与regular两半

factor(q,w)：先把spread w转F32。w<=0或quiet q!=0时返回1；w>=R32(0.2)且q=0时返回正零；其余theta=(min(w,R32(0.2))⊗R32(pi))/R32(0.4)，每步F32分别舍入。factor=R32(R32(cos64(theta))×R32(cos64(theta)))。cos64定义为数学cos结果最近偶数舍入到binary64，再转F32；当前研究接受模型使用系统数学实现，原生最后ULP及所有输入域未单独证明。

每tail当前regular half为raw X⊗当次factor；若F04A.2 mute reset成立，先以正零代替X。previous regular half取本callback处理前raw history，乘同一个当次factor；mute reset成立时先以正零代替旧history。不是复制上callback已经缩放的输出，也不是用上callback factor。

## F04A.4 扩散实例与prefix累加

tail仅在object diffuse非零、spread>0且quiet=0时拥有活动四路滤波实例。inactive的旧实例删除，同时丢弃其previous filter outputs；重新活动时从冷filter state处理previous PCM||current PCM的3072样本，每路输出拆为前/后1536。连续活动时只处理current PCM1536，previous half来自该实例上次保存的四路输出。完整filter数学、构造接口和首块淡入见F04F；DC/DP分别由F04FM/F04FCM精确生成，原研究表不复制。不能把其他layout或未核验合成配置当作缺省。

prefix有11个1536样本累加row，previous/current两组每callback都从正零重新开始。F04G定义gain producer的11点spread、sin²及front补偿，必要参数供应仍未解决。producer给出每tail完整11项finite F32 gain；只对非quiet、spread>0且gain>0并且gain>=R32(1e-6)的成员生成调用，等号包含。按tail递增、prefix output递增遍历，重复(tail,output)成员拒绝。缺失的gain项不能从未出现调用推定为0。执行的output索引组成F03R.4的writer_set，不能用累加后的样本是否为零替代。

每prefix output需mode[11] I32及filter_lane[11] I32参数，lane为-1或0～3。F04FM提供当前11项配对/循环生成规则，22个整数与选定参数精确一致。接口按output索引读取同位置mode/lane，无插值或重排；diffuse非零且lane>=0时使用filtered branch，其余为direct branch。原研究表不随包交付，其他layout尚未定义；不能用全零mode或全lane=-1绕开扩散算法。

direct：对每sample，output=(source PCM⊗gain)⊕old_output。

filtered：A=R32(sqrt64(R64(0.85)))⊗gain，B=(R32(sqrt64(R64(0.15)))⊗R32(mode))⊗gain；output=((source PCM⊗A)⊕old_output)⊕(filter_lane_sample⊗B)。sqrt64为数学sqrt最近偶数binary64结果；每个标示F32乘加分别舍入，不融合。

previous/current两组分别用previous/current PCM及对应filter half，以相同当次gain、branch、mode和调用顺序重新累加。previous prefix不是旧prefix缓存，也不通过当前总和相减恢复。

## F04A.5 48×3072输入窗口

current48为11 prefix rows后接37 regular rows；previous48按同样顺序。每row拼previous1536||current1536形成3072，再逐sample乘F04A.2的同一W，产生postwindow48。

每callback结束提交当次raw PCM到previous PCM、regular raw history到37 history、当前48row到previous current，并提交当次活动filter state/outputs。previous current的保存不改变F04A.3/4已定义的重新计算previous half规则。

## F04A.6 正权矩阵与destination

输入为48×3072 finite F32和逻辑n×48系数，n=0～15；active destinations为n个互异0～14索引，按安装后的active行递增排列。所有15 output rows先清正零。按逻辑row递增、source column0～47递增、sample0～3071递增累加。

系数c<=0时跳过（包括两种零及负值），不把负系数当作相减。c>0时每sample执行p=source⊗c，再output_destination=old_output⊕p。无FMA、无并行归约重排，inactive rows保持正零。n=0不读取不存在的系数。

输入source视为本次调用不可变快照；不能通过共享destination覆盖尚未消费的source。低层数值primitive可验证重复destination累加，但本选定dynamic接口要求active map互异，不能以低层接受范围外推到上层。物理receiver别名关系仍需receiver条款补齐。

## F04A.7 Overlap和1472+64 source ring

每个component row有overlap H[1536]和ring R[1472]。当次矩阵Y[3072]的head与旧H逐项相加：O[i]=Y[i]⊕H[i]；next_overlap复制Y[1536:3072]原始位模式。

O的1536样本先经F04A.8及参数producer得到filtered Z，再生成Type10 component=旧R||Z[0:64]，next_ring复制Z[64:1536]。ring拼接及history复制不做浮点运算。

实现可preview overlap供filter输入，但preview不提交ring/overlap；最终仍从同一个callback开始状态提交一次。不能把preview.next_state再作为commit输入，否则会推进两次。未提供独立filtered Z的数值primitive使用O作history_input，只属于原语测试分支。

## F04A.8 64样本filter及参数代次

每callback按block0～23处理，单block含15×64 source样本。共享parameter producer消费完整15路的本block peak及旧parameter state，先生成next parameter state。filter读取next_state中的previous_gain和smoothed_gain，output_level取block开始的旧state，gain_scale取F04P.1的只读profile。

必要kernel K[64]为finite F32。alpha=old_output_level⊗next_previous_gain，beta=next_smoothed_gain⊗profile.gain_scale。对每row、sample j，读取旧filter history h：a=K[j]⊗(h⊗beta)，b=K[63-j]⊗(h⊗alpha)，intermediate=(a⊕b)⊗256。此时当前source样本只被复制到next filter history，不参加该block输出；形成64-sample history delay。

随后clamp intermediate到[-limit,+limit]，再乘scale=256⊗profile.gain_scale；limit取只读profile.clamp_limit，要求finite且>0。clamp按先min(+limit)、再max(-limit)执行，N00规定相等/零符号。15路各自history冷正零，按block连续推进，不能跨row共用。

输出24块拼接为15×1536 Z，交F04A.7提交ring。kernel、parameter producer的全部非秘密系数/阈值/冷状态必须另行给出；缺失时PARAMETER_UNAVAILABLE，不将本数学网络当作已具备全部生产参数。

## F04A.9 Type10与剩余边界

Type10为16 rows，每row1536 F32：row0为独立direct LFE ring输出，rows1～15依次为component0～14。Type10是receiver输入，不是最终codec六路；最终[ L,R,C,LFE,Ls,Rs ]来自receiver，不在这里用简单6-row挑选代替。

reset清除本文件所有音频history/filter map/component ring，同时按参数producer/receiver的完整constructor重置其状态。EOF按F02四次cached padded输入继续执行本网络，不把cached flush改成全零。shape/type/finite错误按I03；native失败中间提交状态仍未证明。
