# 快速开始

安装 Rust 和 Cargo 后，安装 CLI：

```sh
git clone https://github.com/chyinan/JOCForge.git
cd JOCForge
cargo install --path crates/jocforge-cli --locked
```

RIFF/WAVE 输入使用 48 kHz、每个采样占 3 字节的 24 位整数 PCM。既支持普通 PCM，也支持 PCM 子类型的 `WAVE_FORMAT_EXTENSIBLE`（24 位容器、24 个有效位）；轨道关联仍以 ADM 元数据为准。

将 48 kHz ADM BWF 编码为默认 1024 kbps 的 5.1 音轨。JOCForge 要求绝对路径；`$PWD` 在 PowerShell 和 POSIX shell 中均可用：

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/output.ec3"
```

装有 OpenJOC 时，检查编码结果：

```sh
openjoc inspect output.ec3 --emdf
```

Flat 7.1 默认 1664 kbps，输出需放入新的空目录并使用 `.candidate.ec3` 后缀：

```sh
mkdir flat7x
jocforge encode "$PWD/input.adm.wav" "$PWD/flat7x/mix.candidate.ec3" --joc-layout flat7x
```

![JOCForge CLI](assets/cli-help.png){ .jocforge-hero }

布局、子流、承载和容器参数写在编码指南里。`jocforge encode --help` 会列出这些参数。
