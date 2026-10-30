# F04FM Diffusion 系数与 prefix 配对映射生成

本文件提供 D11-DC、D11-PM/PL 的明确数学生成规则。适用范围为当前四级四路 filter 和固定 11 个 prefix 的顺序；不推定其他 layout。原研究表未随包复制。当前交付仅供内部研究和内部实现，不公开或向第三方分发。

## F04FM.1 四阶 Hadamard 与 all-pass 系数

stage s、lane l 均为 0～3。设 H[s,l]=(−1)^popcount(s AND l)，即按二进制索引的四阶 Sylvester Hadamard 符号。pole=R32(−2/5)，C[s,l]=pole⊗R32(H[s,l])。±1 乘法只改变 pole 的符号，不增加数值舍入误差。

输出按 stage-major、lane-minor 为 16 项 F32，交给 F04F.4 的 coefficient[4][4]。pole 是恢复的行为参数；Hadamard 结构由规则生成，不提供研究侧符号数组副本，也不把数据来源重标为新协议标准。

## F04FM.2 五对 prefix 与中心

选定 prefix 顺序是：pair0 两项、中心单项、pair1/pair2/pair3/pair4 各两项，总计 11 项。每对内部先左成员、后右成员。此处配对是已声明的输出顺序，不按样本、source ID 或音频非零判断重排。

pair p=0～4 选择 filter lane=p mod4；左 mode=+1、右 mode=−1。中心 output=2 的 mode=0、lane=−1，按 F04A.4 进入 direct branch。生成 mode[11] 和 lane[11]，每项 I32，与 prefix output 同索引。

改变 pair 顺序、中心位置或 prefix 数量时，本生成路线报 LAYOUT_DOMAIN。不得把此循环映射泛化为所有扬声器布局。

## F04FM.3 数据与等价依据

实际生成数据为 `../parameters/diffusion-mathematical-v1.json`；核验为 `../test-vectors/DIFFUSION-MATH-GENERATION-VERIFICATION.json`。16 个 coefficient 位模式、11 个 mode 和 11 个 lane 全部与选定消费参数相同，差异 0。

在其余构造配置保持一致的条件下，4 次连续处理（首次 3072、后续各1536样本）的四路输出及完整 next_state 一致。输入为明示有理数规则生成的合成 PCM，不是歌曲或原生缓存。该比较未交付其余研究构造参数，不能当作独立可运行的完整 filter constructor fixture。

等价声明仅为上述固定参数身份和同一状态转移的替换：所有其余输入/配置/状态相同，原参数与生成参数逐位相同，因此同一运算和分支次序的结果相同。这个结论不依赖近似误差容限；D11-DP另由F04FCM生成，本条款不关闭geometry、原始width producer、失败提交或新ADM整链验收。

## F04FM.4 安装与重置

programme 初始化时生成一次，检查输出数量、顺序、类型及参数文件身份，并作为不可变参数安装。reset重建filter动态状态，不改变本生成映射或复用上一programme的状态。生成失败按 I03停止；不从研究表、未知布局或歌曲数据补值。

独立局部复审已通过生成值、输出及规范状态摘要，见DIFFUSION-COMPLETE-LOCAL-REVIEW-STATUS.json；原研究参数对照不能在未提供原值的隔离包内重做。材料与局部核验通过不自动代表整包 `IMPLEMENTATION_READY=YES`。
