# 架构设计

状态：ARCHITECTURE_REVIEW=APPROVED；REPOSITORY_IMPLEMENTATION_READY=YES_FOR_SELECTED_PROFILE。

本仓库正在独立物化批准设计。`jocforge-application::encode` 仍返回 BackendUnavailable；CLI 另有 selected full-file pipeline，可读取 ADM BWF 并为六档显式码率生成 elementary EC-3 输出。新增 `encode --job-xml` 接受严格的 ADM BWF / EC-3 子集，输入 XML 可用 `adm` 或 `atmos_mezz` 分支并保留分支身份；`--preflight-only` 在 PreparedJobInput admission 后停止，不启动 Meter 或输出。MP4/DASH 与未闭合选项显式拒绝。CLI 输出、XML parser acceptance 和选项的音频效果是不同证据层级；DEE parity、独立实现验证和 W04 产品验收仍未完成。帮助和版本输出保留。

采用五个小 crate：domain、application、io、CLI 和 validation。生产入口仅依赖 application，再依赖 domain。io 和 validation 目前是空壳；io 不能反向驱动 domain。未来 CLI 负责 gather/process/persist，application 保持纯协调，通过类型化输入和 staged result 连接 domain。

模块责任见 MODULE-MAP，状态约束见 STATE-OWNERSHIP，流程见 DATAFLOW。Profile 维度见 PROFILE-DIMENSIONS，公共边界见 PUBLIC-API，决策见 ARCHITECTURE-DECISIONS。权威规格包已提供 GTC、owner projection、F01～F15 条款、时钟和 selected profile 功能规格。本实现只使用批准包内材料，不从旧实现猜测时钟、owner、表或算法。

架构批准允许按既有 selected 5.1 profile 分阶段实现，并另行保留 Atmos 7.1 flat7x/1664 的 additive profile contract；该新增 contract 尚非 production-enabled（`production_allowed=false`）。GTC source-role projection、native D0 audio algorithm 和 all-AU EMDF fit 仍未闭合。独立实现验证、W04 产品验收和用户试听仍未运行。禁止从可编译或测试通过推导功能完成。
