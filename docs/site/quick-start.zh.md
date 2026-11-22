# 快速开始

安装 Rust 和 Cargo 后，安装 CLI：

```sh
git clone https://github.com/chyinan/JOCForge.git
cd JOCForge
cargo install --path crates/jocforge-cli --locked
```

将 48 kHz ADM BWF 编码为默认 1024 kbps 的 5.1 音轨：

```sh
jocforge encode input.adm.wav output.ec3
```

装有 OpenJOC 时，检查编码结果：

```sh
openjoc inspect output.ec3 --emdf
```

Flat 7.1 默认 1664 kbps，加上 `--joc-layout flat7x`：

```sh
jocforge encode input.adm.wav output-7.1.ec3 --joc-layout flat7x
```

![JOCForge CLI](assets/cli-help.png){ .jocforge-hero }
