# 架构设计

状态：ARCHITECTURE_REVIEW=APPROVED；REPOSITORY_IMPLEMENTATION_READY=YES_FOR_SELECTED_PROFILE。

本仓库正在独立物化批准设计。`jocforge-application::encode` 仍返回 BackendUnavailable；CLI 另有 selected full-file pipeline，可读取 ADM BWF 并为六档显式码率生成 elementary EC-3 输出。新增 `encode --job-xml` 接受严格的 ADM BWF / EC-3 子集，输入 XML 可用 `adm` 或 `atmos_mezz` 分支并保留分支身份；`--preflight-only` 在 PreparedJobInput admission 后停止，不启动 Meter 或输出。MP4/DASH 与未闭合选项显式拒绝。CLI 输出、XML parser acceptance 和选项的音频效果是不同证据层级；DEE parity、独立实现验证和 W04 产品验收仍未完成。帮助和版本输出保留。

采用五个小 crate：domain、application、io、CLI 和 validation。生产入口仅依赖 application，再依赖 domain。io 和 validation 目前是空壳；io 不能反向驱动 domain。未来 CLI 负责 gather/process/persist，application 保持纯协调，通过类型化输入和 staged result 连接 domain。

模块责任见 MODULE-MAP，状态约束见 STATE-OWNERSHIP，流程见 DATAFLOW。Profile 维度见 PROFILE-DIMENSIONS，公共边界见 PUBLIC-API，决策见 ARCHITECTURE-DECISIONS。权威规格包已提供 GTC、owner projection、F01～F15 条款、时钟和 selected profile 功能规格。本实现只使用批准包内材料，不从旧实现猜测时钟、owner、表或算法。

架构批准允许按既有 selected 5.1 profile 分阶段实现，并另行保留 Atmos 7.1 flat7x/1664 的 additive profile contract；该新增 contract 尚非 production-enabled（`production_allowed=false`）。GTC source-role projection、native D0 audio algorithm 和 all-AU EMDF fit 仍未闭合。独立实现验证、W04 产品验收和用户试听仍未运行。禁止从可编译或测试通过推导功能完成。

当前另有一个明确限定的 Flat7x 部分实现：CLI 可选项 `--joc-layout flat7x` 在 1024 kbps / 5.1 core path 上选择 immutable `JocReconstructionLayout::Flat7x`，由现有 JOC-header writer 发出 ETSI `joc_dmx_config_idx=1` 的 3-bit 字段。省略此项保留 selected 5.1 profile；原有 downmix control、输入解析、audio-generation、topology 和 publication 行为沿用既有配置。完整 Flat7x 文件与官方演示结构验收仍为 `NOT_RUN`。

1664 kbps I0/D0 目标另有 `Flat7xI0D0TargetPlan` 的 value-only 建模、fail-closed metadata check 和 I0→D0 byte-sequence candidate。后者只拼接调用方提供且逐帧 CRC 通过的 bytes，不解析 substream syntax，也不进入 production resolver、frame writer 或 publication flow；`production_allowed=false` 与 D0 audio algorithm、ADM role join、native writer owner、all-AU EMDF fit 的开放状态保持不变。

The additive `codec::i0_frame_assembler::assemble_flat7x_i0_ac3_frame` composes the approved AC-3 header with caller-owned BSI-tail and six audio-block spans, carries optional auxdata, and generates CRC1/CRC2 with zero-residual receipts on synthetic bytes. It creates no audio or source-generation join; `production_allowed=false`. The selected 5.1 E-AC-3 path stays unchanged.

F15 另有独立的 Flat7x AU staging owner value，可为一个 I0+D0 aggregate 建立 proposal、pair-aware ledger entry 与 caller-supplied receipt candidate 验收。该 owner 尚未接入 GTC 或产品路径；5.1 `OutputOwner` 保持独立。

单独的 IO adapter 已可把此 proposal 写到调用方持有的 staging file，执行 sync 和精确范围 readback 后返回 receipt candidate。失败时由调用方废弃 staging attempt；adapter 不做 rollback、rename 或 final exposure，也不连接 encode entry。
