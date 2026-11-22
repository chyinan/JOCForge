# 子流与信令

## 多依赖子流与 LFE 归属

`--dependent-count N` 指定每个 1536 采样访问单元内的依赖子流数。idx0、idx3 可选 0–5，idx1、idx2、idx4 可选 1–6。显式指定流数或 LFE 归属时，JOCForge 使用原生 E-AC-3 I0 和连续编号的 D0…Dn，每帧 6 个音频块。最后一个依赖子流承载严格 ETSI JOC/OAMD，零依赖时由 I0 承载。节目总码率在这些子流之间分配。常规默认输出继续使用原来的编码路径。

| `--lfe-owner` | I0 | D0 | 解码后的 LFE |
|---|---|---|---|
| `independent`（默认） | 有 | 无 | 取 I0 |
| `dependent` | 无 | 有 | D0 补充 |
| `replace-independent` | 有 | 有 | D0 替换 I0 |
| `none` | 无 | 无 | 无 |

单独指定 LFE 时，五载体布局默认使用零个依赖子流，七载体布局默认使用一个。需要 D0 承载 LFE 时至少使用一个。依赖声道映射按 profile 的规范组顺序分配，后环绕对或高度对保持成对。文件输入和 XML 作业使用相同选项；七载体布局的输出仍使用独立空目录中的 `.candidate.ec3`。

```powershell
# 5.X：五个依赖子流，D0 的 LFE 替换 I0，D4 承载 JOC。
jocforge encode "$PWD/input.adm.wav" "$PWD/mix.ec3" --joc-layout idx0 --dependent-count 5 --lfe-owner replace-independent
# 5.X+2：六个依赖子流，D0 补充 LFE，D5 承载高度对和 JOC。
mkdir height-multi
jocforge encode "$PWD/input.adm.wav" "$PWD/height-multi/mix.candidate.ec3" --joc-layout idx4 --total-rate 1664 --dependent-count 6 --lfe-owner dependent
```

## 短块划分

`--block-partition` 支持 `3+3`、`2+2+2` 和 `1x6`（也可写作 `1+1+1+1+1+1`）。默认值是 `6`。短块使用原生 E-AC-3，每个 1536 采样区间的总码率与时长按原参数计算。每个子区间先写 I0，随即写其 D0…Dn；同一组的块数一致。最后一个子区间的最后一条子流承载一次 JOC/OAMD。过低码率若容不下全部同步帧头、指数和元数据，会返回容量错误。

文件输入和 XML 作业共用该选项，可与 `--dependent-count`、`--lfe-owner` 配合。例如：

```powershell
mkdir height-short
jocforge encode "$PWD/input.adm.wav" "$PWD/height-short/song.candidate.ec3" --joc-layout idx4 --total-rate 1664 --block-partition 3+3 --dependent-count 2 --lfe-owner dependent
```

## EMDF 承载位置与信令

`--carriage auxdata`（主分支拼写为 `etsi`）将受保护的 EMDF 容器写入最后一个依赖子流的帧尾。短块划分时，每个 1536 采样区间只在最后一组写入一次。默认使用一个依赖子流；显式指定流数时，帧尾模式要求至少一个。音频块内的承载使用 `--carriage skipfld`。

`--emdf-profile etsi` 和 `--emdf-profile deployed` 分别选择严格 ETSI 信令和部署兼容信令，两种承载都可以搭配任一信令。原有 `--etsi-strict` 仍可用。单独指定帧尾承载时，默认使用严格信令。文件输入和 XML 作业共用这些参数。

```powershell
mkdir aux-strict, aux-deployed
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-strict/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile etsi
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-deployed/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile deployed
```

## 五种 profile 的 Legacy AC-3 核心

`--carriage legacy-ac3` 为 idx0 至 idx4 生成固定 **1280 kbps** 的裸码流。每个 1536 采样区间先写 640 kbps 的 48 kHz AC-3 I0，再写 640 kbps 的 E-AC-3 D0，各 2560 字节。I0 提供可单独播放的 3/2 声床；D0 在帧尾承载 JOC/OAMD，默认使用严格 ETSI 信令。显式添加 `--emdf-profile deployed` 即可使用部署兼容信令，适用于全部五种 profile 和四种 LFE 归属。idx0、idx3 的 D0 替换 L、C，idx1 的 D0 补充后环绕对（0x0200），idx2、idx4 的 D0 补充前高度对（0x0010）。LFE 由 D0 补充或替换时，通道映射另加 0x0001。该模式固定六块、一个依赖子流，LFE 默认由 I0 携带，也支持既有的四种 `--lfe-owner` 取值。

```powershell
jocforge encode "$PWD/input.adm.wav" "$PWD/idx0-legacy.ec3" --joc-layout idx0 --carriage legacy-ac3 --total-rate 1280
jocforge encode "$PWD/input.adm.wav" "$PWD/idx3-legacy.ec3" --joc-layout idx3 --carriage legacy-ac3 --total-rate 1280
```

七载体布局仍使用独立空目录中的 .candidate.ec3 输出。例如先创建 height-core 目录，再运行：

```powershell
jocforge encode "$PWD/input.adm.wav" "$PWD/height-core/song.candidate.ec3" --joc-layout idx4 --carriage legacy-ac3 --total-rate 1280 --lfe-owner dependent
```

XML 作业的 `data_rate` 填 D0 的 `640`。可用 `--job-xml job.xml --joc-layout idx0 --carriage legacy-ac3 --preflight-only` 核对总码率和每个访问单元的长度。
