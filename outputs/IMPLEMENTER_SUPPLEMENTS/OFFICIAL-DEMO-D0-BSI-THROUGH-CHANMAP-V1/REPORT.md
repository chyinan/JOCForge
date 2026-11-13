# 官方演示样本 D0 BSI 到 chanmap

## Wash 核验

用户指定的 MKA 输入 SHA-256 已由 Wash 重新计算，并与 Wash 收据一致。收据通过普通模式和 `python -O` 检查；8 个本地/共享文件逐一哈希一致。实现补充包只包含字段值、bit offset 和哈希收据，不包含 MKA、原始 frame、解析器或音频载荷。

首个 packet 为 I0 后接 D0。I0 从文件 byte 5424 开始；D0 从 byte 7984 开始。样本 D0 是 640 kbps / 2560-byte frame，BSI 到 frame-local bit 76 处观测到 `chanmape=1`、`chanmap=0x1A00`。字段顺序、宽度与绝对 bit 范围见 `FIELD-VALUES.json`。

## 实现使用边界

此包提供官方样本的一条 D0 BSI 输入向量。D0 `frmsiz=1279` 是 640-kbps 样本值；Flat7x 目标的 D0 1024 kbps / 4096 bytes / `frmsiz=2047` 取自独立的 `FLAT7X-D0-RATE-FRMSIZ-TARGET-V1` 契约。

样本的 `dialnorm=31`、`compre=1`、`compr=1` 是该样本实际字段值。它们可作为字段序列检查向量；本包不把这些值定义成任意 ADM/XML 的通用 DRC 默认值，也不提供音频算法或 DRC 映射。`chanmap=0x1A00` 是被观测的 16-bit 数值字段；ADM 行/声道角色映射仍由独立规则处理。

## 状态

D0 BSI 到 `chanmap` 的样本向量已绑定；`mixmdate` 及后续字段未读。此收据不生成 D0 audio spans、完整 frame、AU 或输出文件。
