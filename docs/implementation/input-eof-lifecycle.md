# Selected input and EOF lifecycle observation replay

This is a test-only lifecycle replay for one hash-bound selected run. No lifecycle module or API is exported from the application library, and `encode()` remains unchanged. The replay coordinates already-observed scalar events with the independently tested carrier planner/gate projections; it is not a production input adapter, renderer, audio path, ID14 parser, or JOC body producer.

## Selected call sequence

- Four external process calls each carry 38 source-index/control-position records. Their API positions are 0, 1536, 3072, and 4608. `ExternalControlPosition` is not an ADM/WAV absolute offset, and the control-position values are not converted into samples or byte offsets.
- External calls 1 and 2 run renderer work with observed span 0 and do not call frontend. Calls 3 and 4 run renderer work with observed span 1536 and call frontend source invocations 1 and 2.
- Five flush calls follow. Calls 1 through 4 are active: each invokes one renderer work step, one frontend source step, publishes 4096 bytes, and decrements `flush_remaining` only after the child step returns. Call 5 is empty: it returns 0 with length 0 and no renderer/frontend/planner work.
- The six frontend source calls use the Wash-supplied planner-request lengths `0, 0, 422, 316, 320, 316` and pending ID14 payload extents `132, 326, 222, 226, 222, 218`. These are diagnostic observation inputs, not model-produced body content. The first two are from external calls 3 and 4; the last four are from active flush calls 1 through 4.

The renderer progress and frontend publication-gate progress are independent warmup accumulators. The latter ends at 4608; neither value is treated as an absolute source clock. Published AU ordinals 0 through 3 correspond to flush calls 1 through 4 and remain joined to carrier/JOC producer invocations 2 through 5 by the separate carrier fixture.

## Selected reader/source sample coordinates

The reader/source join covers one selected item only: selection index 0, with source-sample coordinates 2,880,000 through 2,886,000. For each input API call, the selected-segment-relative control position equals the reader cursor before that frame. The test replay derives the selected source sample coordinate by adding that relative cursor to the selected item start:

| Input API call | Relative control / reader cursor before | Derived selected source sample | `read_float` requested / actual | Returned packet samples |
| --- | ---: | ---: | ---: | ---: |
| 0 | 0 | 2,880,000 | 1,536 / 1,536 | 1,536 |
| 1 | 1,536 | 2,881,536 | 1,536 / 1,536 | 1,536 |
| 2 | 3,072 | 2,883,072 | 1,536 / 1,536 | 1,536 |
| 3 | 4,608 | 2,884,608 | 1,392 / 1,392 | 1,536 |

The last returned packet reports a full 1,536 samples although the reader returns 1,392 actual samples. The reader/source capture exports no PCM values or padding words. A separate provider supplement establishes only that the 144-word-per-channel input tail is positive zero; it does not supply the missing sample values. The selected item constructor and CompositeMaster descriptor match by selection index and field values; pointer alias is not proved. These coordinates do not establish an absolute file byte offset, host clock, or general seek/resampling rule.

## EOF, close, and reset boundary

After the fourth active flush, the scalar projection retains gate progress 4608, planner counter 6/first block 2, `flush_started=1`, `flush_remaining=0`, and one pending ID14 descriptor/body extent of 218 bytes. A post-flush process early return is raw U32 `4294967293` / I32 `-3`, consumed flag 1, output length 0, and no state transition. Empty flushes with started flag 0 and 1 both return 0/length 0 while preserving the flag.

Close returns 0 and removes the JOC runtime without semantically clearing those scalar fields. Close returns a pre-retirement scalar projection; it does not retain descriptor/body bytes or read memory after workspace retirement. Same-owner reset/recreate remains `NOT_RUN`; no reset operation is implemented or inferred.

## Production boundary and evidence

`crates/jocforge-application/tests/support/input_eof_lifecycle_replay.rs` is compiled only into the lifecycle test harness. Its external control records, reader/source joins, renderer spans, planner request lengths, and pending byte extents are hash-bound observation/vector inputs. They are not exposed as production API inputs or presented as model-owned output. Production transport still lacks the model-owned protected-body/provider receipt required for end-to-end wiring.

The 15-file Wash handoff manifest SHA-256 is `ef7fa983440fd11784fe9f24f64284240f00fd053853dc1cc28067d3b1cf0d42`; the handoff check SHA-256 is `70c91dcf7c42ae0c57184af165d60df65de49482445bb22e1e9584011174ad51`. Wash replay recorded four process calls, five flush calls, four active flushes, six frontend calls, four published AUs, and zero mismatches. It also records `PCM_values_read=false`, `absolute_file_offset_bound=false`, `same_owner_reset_executed=false`, and `whole_F13C_ready=false`.
