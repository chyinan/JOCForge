# 子流与信令

I0 承载独立呈现。D0 和后续依赖子流可以补充或替换其中的声道。

| 选项 | 取值 | 作用 |
|---|---|---|
| `--dependent-count` | idx0、idx3 为 0 至 5；idx1、idx2、idx4 为 1 至 6 | 设置依赖子流数量。后环绕对和高度对保持成对。 |
| `--lfe-owner` | `independent`、`dependent`、`replace-independent`、`none` | 使用 I0 的 LFE，由 D0 补充，用 D0 替换 I0 的 LFE，或不要 LFE。D0 承载 LFE 时至少要有一个依赖子流。 |
| `--block-partition` | `6`（默认）、`3+3`、`2+2+2`、`1x6` | 把一个 1536 采样区间分成等长的组。`1x6` 也可以写成 `1+1+1+1+1+1`。 |
| `--carriage` | `skipfld`、`auxdata`（别名 `etsi`）、`legacy-ac3` | 把 EMDF 放进音频块，放在帧尾，或使用 legacy AC-3 核心。 |
| `--emdf-profile` | `etsi`、`deployed` | 选择严格 ETSI 信令或部署兼容信令。这个选项与承载位置互相独立。 |

显式指定依赖子流数或 LFE 归属时，编码器使用原生 E-AC-3。这时的码流是 I0 加 D0 至 Dn，每帧 6 个音频块。最后一个依赖子流承载 JOC/OAMD。没有依赖子流时，由 I0 承载。节目总码率在这些子流之间分配。两个选项都不指定时，仍走原来的编码路径。

只指定 LFE 归属时，五载体布局默认 0 个依赖子流，七载体布局默认 1 个。七载体裸码流仍要放进单独的空目录，并使用 `.candidate.ec3` 后缀。文件输入和 XML 作业使用同一组选项。

| `--lfe-owner` | I0 的 LFE | D0 的 LFE | 解码后的 LFE |
|---|---|---|---|
| `independent`（默认） | 有 | 无 | 取 I0 |
| `dependent` | 无 | 有 | D0 补充 I0 |
| `replace-independent` | 有 | 有 | D0 替换 I0 |
| `none` | 无 | 无 | 无 |

5.X 使用五个依赖子流。D0 的 LFE 替换 I0，D4 承载 JOC：

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/mix.ec3" --joc-layout idx0 --dependent-count 5 --lfe-owner replace-independent
```

5.X+2 使用六个依赖子流。D0 补充 LFE，D5 承载高度对和 JOC：

```sh
mkdir height-multi
jocforge encode "$PWD/input.adm.wav" "$PWD/height-multi/mix.candidate.ec3" --joc-layout idx4 --total-rate 1664 --dependent-count 6 --lfe-owner dependent
```

## 块划分

默认值 `6` 表示整段 1536 采样是一组。`3+3`、`2+2+2` 和 `1x6` 把这个区间分成等长的组。时长和总码率仍按原来的 1536 采样计算。每一组先写 I0，再写这一组自己的 D0 至 Dn。最后一组的最后一条子流写一次 JOC/OAMD。码率容不下同步帧头、指数和元数据时，编码返回容量错误。

```sh
mkdir height-short
jocforge encode "$PWD/input.adm.wav" "$PWD/height-short/song.candidate.ec3" --joc-layout idx4 --total-rate 1664 --block-partition 3+3 --dependent-count 2 --lfe-owner dependent
```

## EMDF 承载与信令档

`--carriage skipfld` 把受保护的 EMDF 容器写进音频块。`--carriage auxdata` 把它写到最后一个依赖子流的帧尾。短块划分时，每个 1536 采样区间只在最后一组写一次。帧尾承载至少需要一个依赖子流。未指定流数时默认一个，并且默认使用严格信令。

`--emdf-profile etsi` 选择严格 ETSI 信令。`--emdf-profile deployed` 选择部署兼容信令。两种承载都可以搭配任一档信令。`--etsi-strict` 仍可用于原生 5.1 和 Flat 7.1。idx0、idx2 和 idx4 默认已经是严格信令。

```sh
mkdir aux-strict
mkdir aux-deployed
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-strict/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile etsi
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-deployed/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile deployed
```

## Legacy AC-3 核心

`--carriage legacy-ac3` 为 idx0 至 idx4 写出固定 1280 kbps 的裸码流。每个 1536 采样区间先写一条 640 kbps、48 kHz 的 AC-3 I0，再写一条 640 kbps 的 E-AC-3 D0。每条子流 2560 字节。这个模式固定为六块、一个依赖子流。I0 是可以单独播放的 3/2 声床。D0 在帧尾承载 JOC/OAMD，默认使用严格 ETSI 信令。加上 `--emdf-profile deployed` 即改为部署兼容信令。四种 `--lfe-owner` 都可以用，默认是 `independent`。

idx0 和 idx3 的 D0 替换 L 和 C。idx1 的 D0 补充后环绕对（`0x0200`）。idx2 和 idx4 的 D0 补充前高度对（`0x0010`）。D0 补充或替换 LFE 时，通道映射再加上 `0x0001`。

XML 作业里把 D0 的 `data_rate` 填为 640。`--preflight-only` 会报告总码率和每个访问单元的长度。

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/idx0-legacy.ec3" --joc-layout idx0 --carriage legacy-ac3 --total-rate 1280 --lfe-owner replace-independent
```

七载体输出仍使用单独的空目录：

```sh
mkdir height-core
jocforge encode "$PWD/input.adm.wav" "$PWD/height-core/song.candidate.ec3" --joc-layout idx4 --carriage legacy-ac3 --total-rate 1280 --lfe-owner dependent
```
