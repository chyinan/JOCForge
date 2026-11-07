# Composite-format count 的 image identity 更正

已封存的 layout-writer、frame-shape 和 composite-count source identities 曾把同一 Atmos DDP 插件镜像的 SHA-256 记成 `3d66bcec…cd1e1cce8b7e95d2`。补充的 Ghidra Program 身份查询和磁盘文件重算都得到 `3d66bcec…cdce1cce8b7e95d2`。Wash 侧再次对该 DLL 计算 SHA-256，也得到相同的 64 位值。

身份更正包复用了原来的只读 Ghidra project/program，并核对了 layout-writer 与 frame-shape 两个 parent runner。原有 sealed parent 和 predecessor 文件保持不变；错误摘要作为历史转录错误保留，不再用作 image identity。

因此，可将 layout-flag writer、composite-format count store 和 frame-shape reader 静态证据归到同一目标镜像。运行时 layout branch、physical CHNA N 到 composite count 的连接、一般轨数范围和音频效果仍未闭合。
