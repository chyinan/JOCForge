# 开发与验证

Windows x64 Release 在 Ryzen 7 9800X3D 上编码一份 191.69 秒、26 声道的 48 kHz 动态 ADM。5.1 / 1024 kbps 首次导出用时 41.4 秒，其中响度测量 16.2 秒；旧版分别为 52.0 秒和 20.0 秒。复用响度时，5.1 / 1024 kbps 用时 25.2 秒，Flat 7.1 / 1664 kbps 用时 37.8 秒。两份整曲 EC3 和首次测量得到的响度、真峰值与门控结果均与旧版逐位一致。

项目采用 Rust 2024 工作区，包含领域逻辑、应用、文件读写、命令行和独立验证五个模块。编码请使用 CLI，应用层的 `encode()` 接口目前返回 `SourcePipelineUnavailable`。命令参数和版本可通过 `jocforge --help` 和 `jocforge --version` 查看。[架构说明](https://github.com/chyinan/JOCForge/blob/main/docs/architecture/ARCHITECTURE.md)

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --locked
python scripts/check-boundaries.py
```

## 本地预览文档

```sh
python -m pip install -r requirements-docs.txt
python -m mkdocs serve
python -m mkdocs build --strict
```

Pages 工作流从 `main` 构建本站。
