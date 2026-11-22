# 开发与验证

这是一个 Rust 2024 工作区，分成领域逻辑、应用编排、文件读写、命令行和独立验证。编码使用 CLI。应用层的 `encode()` 入口目前仍返回 `SourcePipelineUnavailable`。`jocforge --help` 列出参数，`jocforge --version` 打印版本。结构说明见[架构文档](https://github.com/chyinan/JOCForge/blob/main/docs/architecture/ARCHITECTURE.md)。

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --locked
python scripts/check-boundaries.py
```

## 已测量的导出

下面的时间来自一台 Ryzen 7 9800X3D 上的 Windows x64 Release。输入是一段 191.69 秒、26 声道、48 kHz、带动态对象的 ADM。

| 导出 | 用时 |
|---|---|
| 首次 5.1 / 1024 kbps | 41.4 秒，其中响度测量 16.2 秒 |
| 上一版构建的同一次首次导出 | 52.0 秒，其中响度测量 20.0 秒 |
| 命中响度缓存的 5.1 / 1024 kbps | 25.2 秒 |
| 命中响度缓存的 Flat 7.1 / 1664 kbps | 37.8 秒 |

两份整曲 EC-3 与上一版构建逐位一致。首次测量得到的响度、真峰值和门控结果也一致。

5.1 的六档码率（384 kbps 至 1024 kbps）都已在本地完成整文件编码。这些编码通过了输入配置、帧长和无密钥完整性检查。

## 本地预览文档

```sh
python -m pip install -r requirements-docs.txt
python -m mkdocs serve
python -m mkdocs build --strict
```

Pages 工作流从 `main` 构建两种语言。
