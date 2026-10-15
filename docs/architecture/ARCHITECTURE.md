# 架构设计

状态：ARCHITECTURE_REVIEW=PENDING；IMPLEMENTATION_READY=NO。

本基线只建立工程边界。encode 在 application 返回 BackendUnavailable，CLI 映射退出码 2。没有音频读取、编码、状态推进、payload 或发布步骤。帮助和版本输出保留。

采用五个小 crate：domain、application、io、CLI 和 validation。生产入口仅依赖 application，再依赖 domain。io 和 validation 目前是空壳；io 不能反向驱动 domain。未来 CLI 负责 gather/process/persist，application 保持纯协调，通过类型化输入和 staged result 连接 domain。

模块责任见 MODULE-MAP，状态约束见 STATE-OWNERSHIP，流程见 DATAFLOW。尚未收到可信 GTC/owner projection、F01/F12 或完整功能规格。本设计不从旧实现猜测时钟、owner、表或算法。评审需明确这些输入及模块映射，再批准第一阶段实现。

此次可评审项：crate 单向依赖、空壳粒度、错误出口、发布事务边界和新测试来源。音频功能、数值、wire 和验收阈值均未决。禁止从可编译或测试通过推导功能完成。
