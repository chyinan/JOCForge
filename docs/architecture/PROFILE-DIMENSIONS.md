# Profile 维度

Production `EncodingProfile` 由六个正交维度组成。Validation 使用独立 `ValidationProfile`。公共 API 不得把 render target 传入 production resolver。

| 维度 | 类型 | 当前唯一允许值 | 拒绝规则 |
|---|---|---|---|
| Authored input | `AuthoredInputProfile` | ADM BWF；PCM24；48 kHz；实际行数随输入携带 | 输入行数必须与 RIFF PCM channel count 和 CHNA declared track count 一致，并满足选定 ADM graph 约束；不补齐、截断或重排。 |
| Core carrier | `CoreCarrierProfile` | 5.1；六 carrier | 其他布局或 carrier 数返回 `UNSUPPORTED_PROFILE`。 |
| JOC downmix | `JocDownmixProfile` | DEE selected mode0；phase1；index3 | 其他 mode、phase 或 index 返回 `UNSUPPORTED_PROFILE`。 |
| Substream topology | `SubstreamTopology` | 1 个 independent；0 个 dependent | multi-independent 或任意 dependent 返回 `UNSUPPORTED_PROFILE`。 |
| Codec rate | `CodecRateProfile` | 1,024 kbps；6 blocks；1,536 samples/AU；4,096 bytes/AU | 任一字段不同都返回 `UNSUPPORTED_PROFILE`。 |
| EMDF carriage | `EmdfCarriageProfile` | `deployed_compat`（默认）或 `etsi_emdf`（可选） | 未登记模式返回 `UNSUPPORTED_PROFILE`，不得降级。 |
| Validation render target | `ValidationProfile.render_target` | OpenJOC 7.1.4，仅用于 validation | 不参与 production profile resolution。 |

## Supported row

当前只有一行 production profile。`EmdfCarriageProfile` 在同一行内允许两个受审 carriage mode。下表不包含 `RenderTarget`。

| AuthoredInputProfile | CoreCarrierProfile | JocDownmixProfile | SubstreamTopology | CodecRateProfile | EmdfCarriageProfile |
|---|---|---|---|---|---|
| selected ADM BWF / PCM24 / 48 kHz；N 个物理行，10 ≤ N ≤ 256，CHNA=N，10 个 bed + N−10 个 object | 5.1 / 六 carrier | mode0 / phase1 / index3 | single independent / zero dependent | 1,024 kbps / 6 blocks / 1,536 samples/AU / 4,096 bytes/AU | `deployed_compat` 或 `etsi_emdf` |

### 输入行数扩展的边界

以上 ADM 行数扩展是 2026-11-03 用户指示下的输入准备扩展，不重写 `APPROVAL.md` 中记录的 38-channel 历史基线，也不表示 end-to-end encoder 已接受任意行数。当前 36-channel 输入可经 RIFF PCM24 reader、job configuration、ADM graph、object-event admission、converter storage map、primary metadata/control values、selected BSI profile-field projection 和输入侧 physical-row map 保持 36-row cardinality。

现有 F01E normal-control snapshot、meter source bridge 和编码 callback 仍固定为 38 行；36 行输入在这些下游路径仍被拒绝或未接线。ADM/PCM 行数到 DDP native frame-view count 的 same-owner/generation join 仍为 `UNBOUND`，不得把 ADM row count 传入 native frame-view count。没有 36-channel end-to-end encode 结论。

## ValidationProfile

`ValidationProfile` 只包含 `render_target`。改变 `RenderTarget` 不得改变 production `EncodingProfile` 的接受或拒绝结果。

未列入此表的任何组合都返回 `UNSUPPORTED_PROFILE`。不得自动降级，不得切换旧实现，也不得提前加入未来 variant。
