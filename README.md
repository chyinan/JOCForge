<p align="center">
  <img src="docs/assets/jocforge-logo.png" alt="JOCForge" width="360">
</p>

# JOCForge

JOCForge 是一个用 Rust 编写的独立 E-AC-3 JOC 编码器，可将 48 kHz ADM BWF 母版编码为带空间元数据的音轨。支持 5.1、Flat 7.1 和前高度载体布局，可从文件或 XML 作业读取输入。

**[简体中文文档](https://chyinan.github.io/JOCForge/zh/)** · **[English documentation](https://chyinan.github.io/JOCForge/)**

## 快速开始

在本仓库构建 CLI，然后导出默认 5.1、1024 kbps 的 EC-3 音轨：

```powershell
cargo build --release --locked -p jocforge
.\target\release\jocforge.exe encode "D:/Audio/input.adm.wav" "D:/Audio/mix.ec3" --data-rate 1024
```

[布局、码率与对象数](docs/site/layouts.zh.md) · [子流与信令](docs/site/substreams.zh.md) · [CMAF、MKA 与 MP4](docs/site/containers.zh.md) · [XML 作业与响度](docs/site/jobs.zh.md)

[Apache License 2.0](LICENSE) · [第三方声明](THIRD_PARTY_NOTICES.md)
