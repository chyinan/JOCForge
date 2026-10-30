# F03R Source 路由与特征 flags

本条款连接 F04A 的音频窗口与 F03F 的特征状态。它定义选定的 38 轨、11 个 prefix、37 个 tail、48 个 source 分支。其他配置不能套用这个分支。

## F03R.1 输入与顺序

每 callback 的 control 行按输入物理轨道顺序覆盖 0～37，行号互异。输入包括语义 bed/object 分类、direct LFE 判定、mute 0/1、finite F32 width，以及特征 flags 使用的 component_enum 和 diffuse。音频扩散配置与这些特征字段属于不同接口，不能仅因名称相同就共用字段。

选定分支要求恰好一个 direct LFE 行。按 control 输入顺序跳过该行，其余行依次安装到 source 11～47。source 0～10 留给 prefix。LFE 的物理轨号由 ADM 安装结果决定。测试把 LFE 放在不同物理轨，禁止硬编码为参考歌曲轨号。

本条款的控制行到 source 映射只适用已选定的 regular 路由模式。该路由模式与下一节的 selector_mode 是不同配置量。F01E 提供配置安装、对象事件更新、物理控制行和 semantic direct LFE；后续 gain/quiet/assignment producer 仍由本章其他条款及 Q03 跟踪。

## F03R.2 共享 route enable

接口输入 target_capacity 为正整数，selector_configuration_flag 为整数。该 flag 非零时 selector_mode=1；否则 capacity<11 时 mode=0，capacity>=11 时 mode=2。

仅 mode=1 消费另一个整数配置量 other_config6。此时缺少该量报 CONFIGURATION_UNAVAILABLE；other_config6 非零时 route_enabled=false，等于零时为 true。mode=0/2 不消费这个量，route_enabled=true。

选定 layout_enum=8、spatial_config_mode=0 的已验证安装分支得到 capacity=15、prefix_count=11、mode=2 和 route_enabled=true。该分支短路 other_config6；不需要供应其具体值。F01E 从完整固定 job XML 产生这些输入。其他模式不属于选定 profile。route_enabled=false 的音频及 flags 分支尚未恢复，不能把所有 flags 清零后继续输出。本条款对该分支报 UNSPECIFIED_ROUTE_BRANCH，按 I03 停止。

## F03R.3 Tail flags

每个 tail 消费同一 callback 开始时的 raw history H[1536]、当前 PCM X[1536] 和 F04A.2 的 W[3072]。activity 的最早 witness、严格阈值 2^-23、mute 和 history_after 均按 F04A.2。history_after 保存 raw X，包括静音、tiny 与被 width 排除的样本。

定义 activity_flag=quiet，reset_flag=mute。当前恢复分支要求特征 component_enum=0 且 diffuse=0。其他值报 UNSPECIFIED_FEATURE_FLAG_BRANCH；不能静默替换为零。

当 reset_flag=0、activity_flag=0 且 width>=R32(0.2)，将两项都设为 1。其余情况保留初值。门限相等包含在排除分支内；比较前 width 转 F32。width 为负或零不触发这个排除条件。

因此，无 witness、未 mute 的 tail 得到 1/0，F03F 走 carry；有 witness、未 mute 且 width 小于门限时得到 0/0，F03F 消费音频；mute 或活动 tail 达到门限时得到 1/1，F03F 优先 reset。这些 flags 是控制值，不从已经缩放的 regular 音频重新判定。

## F03R.4 Prefix flags 与写入集合

每 callback 开始时，11 个 prefix 的 flags 全部为 1/1。F04A.4 的正 gain 写入调用执行后，把对应 prefix 改为 0/0。输入 writer_set 是本 callback 实际执行的 prefix output 索引集合，元素必须在 0～10。

调用的资格由完整 gain producer、quiet、width 和 gain 门限决定。writer_set 不能从稀疏诊断记录或非零 PCM 推导。重复写入同一 prefix 仍得到 0/0；只有累加顺序影响音频。

即使 PCM 为零、窗端点为零，或多个输入抵消，执行有效写入的 prefix 仍为 0/0。没有写入的 prefix 为 1/1。测试允许给 flags 原语提供非零音频但空 writer_set，以验证 flags 不检查样本；这不是合法音频 producer 的整链实例。

## F03R.5 所有权、错误与验收

音频 activity 与 flags activity 都读取 callback 开始时的同一份 raw history。允许分别计算，但最终提交 raw X 一次。不能先提交音频 history，再把新 history 当作 flags 的旧输入。F04A 的 filter history、previous PCM 与这些 raw history 分别拥有。

输出固定为 source 0～47 顺序的两项整数 flags。验证映射覆盖 37 个 tail、无重复 source、无缺失 history，及输入 finite/type/shape。错误遵循 I03；此处没有证明原生中途失败时的提交行为。

局部核验见 `../test-vectors/SOURCE-FLAGS-LOCAL-VERIFICATION.json`。该文件提供严格阈值、mute history、width 等号、零音频 writer_set、不同 LFE 轨号和配置消费用例。完整选定 control producer、真实 ADM 字段和失败 preimage 见 F01E 的 v2 向量；gain/writer_set 及最终整链审查仍须单独完成。
