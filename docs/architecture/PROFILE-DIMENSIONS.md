# Profile 维度

Production `EncodingProfile` 由六个正交维度组成。Validation 使用独立 `ValidationProfile`。公共 API 不得把 render target 传入 production resolver。

| 维度 | 类型 | 当前唯一允许值 | 拒绝规则 |
|---|---|---|---|
| Authored input | `AuthoredInputProfile` | selected ADM BWF；38-channel PCM24；48 kHz | 其他容器、通道数、位深或采样率返回 `UNSUPPORTED_PROFILE`。 |
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
| selected ADM BWF / 38-channel PCM24 / 48 kHz | 5.1 / 六 carrier | mode0 / phase1 / index3 | single independent / zero dependent | 1,024 kbps / 6 blocks / 1,536 samples/AU / 4,096 bytes/AU | `deployed_compat` 或 `etsi_emdf` |

## ValidationProfile

`ValidationProfile` 只包含 `render_target`。改变 `RenderTarget` 不得改变 production `EncodingProfile` 的接受或拒绝结果。

未列入此表的任何组合都返回 `UNSUPPORTED_PROFILE`。不得自动降级，不得切换旧实现，也不得提前加入未来 variant。
