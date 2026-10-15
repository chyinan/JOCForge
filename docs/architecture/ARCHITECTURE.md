# 架构设计

状态：ARCHITECTURE_REVIEW=APPROVED；REPOSITORY_IMPLEMENTATION_READY=YES_FOR_SELECTED_PROFILE。

本仓库正在独立物化批准设计。encode 仍在 application 返回 BackendUnavailable，CLI 映射退出码 2。当前没有音频读取、编码、状态推进、payload 或发布步骤。帮助和版本输出保留。

采用五个小 crate：domain、application、io、CLI 和 validation。生产入口仅依赖 application，再依赖 domain。io 和 validation 目前是空壳；io 不能反向驱动 domain。未来 CLI 负责 gather/process/persist，application 保持纯协调，通过类型化输入和 staged result 连接 domain。

模块责任见 MODULE-MAP，状态约束见 STATE-OWNERSHIP，流程见 DATAFLOW。Profile 维度见 PROFILE-DIMENSIONS，公共边界见 PUBLIC-API，决策见 ARCHITECTURE-DECISIONS。权威规格包已提供 GTC、owner projection、F01～F15 条款、时钟和 selected profile 功能规格。本实现只使用批准包内材料，不从旧实现猜测时钟、owner、表或算法。

架构批准只允许按 selected profile 分阶段实现。未来 profile 不在当前批准范围。独立实现验证、W04 产品验收和用户试听仍未运行。禁止从可编译或测试通过推导功能完成。
