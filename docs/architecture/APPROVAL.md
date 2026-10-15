# 架构批准记录

批准日期：2026-10-15。

批准的架构基线提交：`469cc61cb4345c89dc96cb205f991ac23c26457b`。

本隔离目录原先没有 Git 历史。本地首次基线提交为 `e3d1e8e56933d04fe10337b6ed7fb0843f3659bc`。该提交只建立批准架构的本地 Git 身份，不替代上面的批准基线引用。

## 权威规格

唯一算法权威路径：

`C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent\outputs\IMPLEMENTER_PACKAGE`

`MANIFEST.json` SHA-256：

`c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`

实现前核验结果：Manifest 包含 405 个受约束条目。全部条目的文件大小和 SHA-256 匹配。缺失和不匹配均为 0。`REVIEW-EXPOSURE.json` 是 Manifest 声明的 detached review exposure，文件内绑定同一 Manifest SHA-256。

## 批准范围

当前批准范围只有 selected profile：

- selected ADM BWF，38-channel PCM24，48 kHz；
- 5.1 六 carrier；
- selected DEE JOC mode0、phase1、index3；
- 1 个 independent substream，0 个 dependent substream；
- 1,024 kbps，6 blocks，1,536 samples/AU，4,096 bytes/AU；
- `deployed_compat` 为默认模式，`etsi_emdf` 为可选模式；
- OpenJOC 7.1.4 只作为产品验收 render target。

其他 profile 不在当前批准范围。实现必须返回 `UNSUPPORTED_PROFILE`，不得自动降级或调用旧 fallback。

## 当前门禁

- `ARCHITECTURE_REVIEW=APPROVED`
- `REPOSITORY_IMPLEMENTATION_READY=YES_FOR_SELECTED_PROFILE`
- `PRE_HANDOFF IMPLEMENTATION_READY=YES`
- `INDEPENDENT_IMPLEMENTATION_VALIDATION=NOT_RUN`
- `PRODUCT_ACCEPTANCE/W04=NOT_RUN`
- `PRIVATE_VENDOR_KEY=EXCLUDED`
- `BIT_EXACT=NOT_REQUIRED`

独立实现验证和 W04 产品验收必须按权威规格单独执行。当前批准不表示这些门禁已通过。
