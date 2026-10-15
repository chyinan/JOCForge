# 状态所有权

权威来源是 `parameters/selected-owner-projection-v1.json`、`specs/00-selected-owner-projection-and-j05.md` 和 GTC。当前 projection 有 7 个 field group、7 个 receipt owner 和 6 个 commit owner。`body_proposals` 是纯值，无 committed state。

每个 owner 的 state 都保持私有。跨 owner 只传 immutable input、proposal、staged result 或 receipt。接收者不得反向写生产者 state，也不得重新执行已产生 receipt 的 stage。

## Owner 登记

| Field group | Functional ID | Owning module | Exact state schema | Cold provider | Clock | Stage authority | Commit authority | Reset authority | EOF/drain | Fatal behavior | Canonical serialization | Receipt type |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `frontend_and_installed48` | F03W | `domain/frontend`、`domain/spatial` | `source_cursor`、`EOF_cache`、`event_control`、`render_diffusion`、`source_flags`、`feature`、`target_state`、`selector_scratch`、`u04_scratch`、`nnls_outer_scratch`、`matrix`、`markers`、`writer_set` | `parameters/adm-installed48-state-v1.json` | `FrontendCallbackIndex` | F03W | `spatial_control` | programme full reset | 4 次 cached drain；提交 EOF cache | discard callback staged root；programme `RETIRED` | `ADM_INSTALLED48_STATE_V1` 字段顺序 | `F03WReceipt` |
| `reader56_targets` | F05R56 | `domain/joc/oamd` | `metadata_history`、`previous_active`、`current_target_snapshot`、`reader_records_16x56`、`target_words_16x6`、`reader_invocations` | `test-vectors/READER56-TARGET-STATE-SCHEMA.json` | callback `g`；reader invocation `q=g-2` | F05R56 | `spatial_control` | programme full reset | 前两 callback 只推进 history；drain 同 callback 域 | discard staged reader/target state；programme `RETIRED` | `READER56_TARGET_STATE_SCHEMA` | `F05R56Receipt` |
| `component_lfe_j05` | F04C | `domain/spatial` | `component_overlap_15x1536`、`component_ring_15x1472`、`filter_history_15x64`、`component_parameters`、`lfe_slots_3x1536`、`lfe_indices`、`receiver_state`、`receiver_audio_qmf`、`fitting_qmf` | `test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json` | callback `g`；receiver `r=g-2` | F04C | `spatial_audio` | programme full reset | component/LFE 每 callback 推进；receiver/QMF 仅 `g>=2` | discard F04C staged root；programme `RETIRED` | `F04_WHOLE_SPATIAL_STATE_SCHEMA` | `SelectedJ05OwnerReceipt` |
| `joc_after_j05` | F05JL | `domain/joc` | `temporal`、`geometry`、`fit180`、`profile_retry`、`huffman`、`id14_writer`、`sequence` | `test-vectors/LIVE-JOC-COORDINATOR-STATE-SCHEMA.json` | dispatch `d=g-1=s+1` | F05JL | `joc` | programme full reset | EOF 丢弃 ID14 pending writer 4248 | six-stage fatal 丢弃完整 JOC staged root；programme `RETIRED` | `JOC_COORDINATOR_STATE_V1` | `F05JLReceipt` |
| `body_proposals` | F05L | `domain/joc/oamd`、`domain/joc/coefficient_payload` | immutable `id11_body`、`id14_body`；无 committed state | 不适用，输入由 F05R56/F05JL receipt 提供 | source `s` | F05L 只生成纯 proposal | `none_pure_value` | 不适用 | proposal 随 source 进入 F13；不拥有 pending queue | 丢弃 proposal；无 state 可回滚 | body canonical bytes | `F05LBodyProposal` |
| `metadata_timeline` | F13 | `domain/joc/emdf` | `id1_timer`、`queue11`、`queue1`、`queue2`、`queue14`、`emdf_assembly` | `parameters/joc-timer-order-v1.json` 与 GTC queue cold schema | source `s`、writer 和 AU join | F13 | `metadata` | programme full reset | source `s` pop JOC writer `s-1`；EOF 丢弃 ordinary 4247/4248 和 ID14 4248 | 四 queue 联合 stage 失败时 committed queues 全不变；programme `RETIRED` | queue before/after generation、writer 和 canonical body 顺序 | `MetadataTransactionReceipt` |
| `codec_and_publication` | F12、F14K、F15P | `domain/codec`、`domain/integrity`、`domain/publication` | `codec_state`、`integrity_receipt`、`staging_stream`、`durable_ledger`、`published_au_count` | `parameters/f07-f12-source-state-schema-v1.json` 与 `parameters/keyless-publication-state-v1.json` | source `s`；published AU `j=s-2` | F12 stage codec；F14K stage integrity；F15 stage output | `codec_and_output` | programme full reset；abandon attempt temp | source0/1 不 append；EOF 无 codec flush；最终 4,247 AU | no-best、CRC、append、fsync 或 replace fatal；committed root 不变，temp abandoned | source state、4,096-byte frame identity 和 durable ledger | `F15PublicationReceipt` |

## Commit owner 统计

| Commit owner | Field groups |
|---|---|
| `spatial_control` | `frontend_and_installed48`、`reader56_targets` |
| `spatial_audio` | `component_lfe_j05` |
| `joc` | `joc_after_j05` |
| `metadata` | `metadata_timeline` |
| `codec_and_output` | `codec_and_publication` |
| `none_pure_value` | `body_proposals`，不属于 committed owner 计数 |

programme root 是顶层协调边界，不是第七个 leaf commit owner。F15 协调一次 source-root commit；leaf executor 每 source 仍只执行一次。

## Receipt 与失败不变量

每个 staged/committed owner receipt 至少包含 `owner_id`、`transaction_id`、before/after generation、before/after canonical identity、`changed` 和 `commit_count`。Hash 不能替代完整 after state。

fatal 时丢弃完整 staged root。除 global status 和 fatal receipt 外，committed owner identity 保持不变。programme 进入 `RETIRED`，后续 stage 为 `NOT_RUN`。恢复销毁整个 owner root，增加 attempt，并从 meter programme 起点重新构造所有 cold provider。
