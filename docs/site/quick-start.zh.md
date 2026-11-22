# 快速开始

安装 Rust 和 Cargo 后，在 PowerShell 里构建独立编码分支。

```powershell
git clone --branch experimental/independent-encoder https://github.com/chyinan/JOCForge.git
cd JOCForge
cargo build --release --locked -p jocforge
```

选择一份 ADM BWF 和新的输出文件名。以下命令生成 1024 kbps 的 5.1 音轨，默认 15 个 JOC 对象。

```powershell
$adm = (Resolve-Path -LiteralPath "D:/Audio/input.adm.wav").Path
$ec3 = Join-Path $PWD "mix.ec3"
.\target\release\jocforge.exe encode "$adm" "$ec3" --data-rate 1024
```

装有 OpenJOC 时，使用 `openjoc inspect "$ec3" --emdf` 查看对象数、码率和元数据。

![JOCForge CLI](assets/cli-help.png){ .jocforge-hero }
