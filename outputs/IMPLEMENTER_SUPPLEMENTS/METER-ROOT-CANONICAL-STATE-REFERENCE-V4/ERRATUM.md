# v3 erratum to the standalone Meter root materialization

This package supersedes the unsealed v3 working copy and corrects v2 without changing its algorithm rules.

1. GTC state_ref now follows the separately approved GTC-OWNER-STATE-REF-CONVENTION-V1 exactly: owner_id meter uses state_ref owners.meter. Snapshot identity and generation remain in their named fields; the member resolves to full root bytes.
2. ROOT-FIELDS contains one entry per root key. The duplicate Meter LFE ring entry is removed.
3. Window queue serialization keeps F06W processed as its own published-sample clock and directly stores every SampleBlockQueueState field plus all pending_sample_words. Transition-time checks reconcile produced spans, queue appends and complete 1024-block consumption; serialization does not derive processed/ring counts from similarly named fields.
4. The canonical local queue key callback_index maps from the implementation-reported runtime next_callback_index. It is distinct from the MeterPcmSourceCursor source-read index and MeterLfeDelayState ring-slot read_index.
5. All three snapshot cases are explicitly synthetic cold roots before callback0. N1537 describes a schedule of two normal callbacks if run, not two callbacks already processed.
6. Queue owner completeness now includes published_samples and last_upstream_generation. The Wash supplied runtime identity is bound by SHA256 8efacd5236f3a56f5621b8fc420bc76dcc99fb4dfd41a5f932d0b9f544c3a315. METER-SAMPLE-QUEUE-LIFECYCLE-V1 binds published_samples to the F06W valid-span publisher update. MeterUpstreamGeneration is a U64 newtype carried as Option: cold None; after each successful callback g it is Some(g+1), including valid=0 and drains; phase is separate. The input-only after vectors include N=9217 terminal generation10 and selected N terminal generation4250.
7. source_samples=N, normal_callback_count Q=ceil(N/1536), and expected_published_samples=Q*1536 are explicitly reconstructible from the canonical binding. They are immutable schedule values, not hidden queue state.

The MeterLfeDelayState and pending queue payloads are complete fields in the root snapshot. DRC and DSP execution scope is unchanged.

8. The canonical key sample_block_queue.pcm_block_fill_cursor maps to runtime SampleBlockQueueState.block_fill_cursor: SampleBlockFillCursor, constrained to U32. The canonical JSON name remains stable; it is not the runtime member spelling. Root bytes for the three cold input-only snapshots are unchanged by this documentation-only mapping correction.
