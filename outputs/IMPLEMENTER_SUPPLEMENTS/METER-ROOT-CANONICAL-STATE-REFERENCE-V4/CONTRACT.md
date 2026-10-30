# Standalone Meter root materialization erratum

This erratum supersedes the v2 representation notes for GTC reference syntax, root-key uniqueness, queue-field canonicalization, runtime field-name mapping and input-only case labels. The original v2 package is unchanged.

## GTC state reference and complete bytes

The approved GTC convention sets the registered Meter owner reference to exactly owners.meter. This is an opaque member name in the complete immutable programme snapshot. Programme identity, attempt, callback generation and state hash remain in their named GTC/root fields; they are not encoded in state_ref.

The meter member resolves to the complete canonical root JSON bytes described in ROOT-FIELDS.json. Those bytes include the prepared input/config identity, independent MeterPcmSourceCursor, all Meter leaves, window queue values and LFE delayed-frame payloads. A digest alone is never the referenced state. The three input-only examples are stored as complete canonical JSON member values so their owners.meter references are concrete.

ROOT-FIELDS contains exactly one entry per canonical root field. The Window and LFE leaves appear once each.

## Window scheduler and SampleBlockQueueState

F06W.1 defines processed as total valid six-channel output samples published after the 3104 startup trim. It increments by each valid output span and finishes at P=Q×1536. It is not a callback count or a complete-block count.

The root stores the F06W processed value and all actual SampleBlockQueueState values. The Wash-identified owner fields are published_samples, last_upstream_generation, pcm_enqueued_samples, pcm_consumed_samples, complete_block_count, block_fill_cursor:SampleBlockFillCursor (U32), startup_remaining, scheduler_pending, next_callback_index and every lane's pending_sample_words. The canonical key callback_index maps from runtime next_callback_index; it counts completed Meter callbacks. It is distinct from MeterPcmSourceCursor.read_index (normal reads) and MeterLfeDelayState.read_index (ring slot0..2). published_samples maps reversibly to the F06W processed sample clock: both advance by the same valid span under the approved lifecycle publisher, including a zero increment when valid=0. Keep both owner/clock fields and check equality at each successful commit; don't infer the mapping from cold equality alone. last_upstream_generation is Option<MeterUpstreamGeneration>, whose selected representation is a U64 newtype: cold=null; after every successful callback g it is Some(g+1), even when valid=0. NORMAL/DRAIN phase is stored separately and is not packed into this generation.

F06W.2 publishes six-channel samples in order and consumes only complete 1024-sample blocks: C=floor(P/1024)×1024. It leaves P−C samples per lane pending. F06W.5 independently accumulates filtered samples into 960-sample windows, leaving C mod960 at EOF. For selected N=6,522,906, P=6,523,392, C=6,522,880, pending PCM=512/lane, complete blocks=6,370, windows=6,794, and energy tail=640/lane. The pipeline passes SCP16 through OAR to six output lanes before this scheduler; it is not a 16-lane energy queue.

Prepared binding contains source_samples=N, normal_callback_count=Q=ceil(N/1536), and expected_published_samples=Q*1536. These schedule values are reversible from N and the fixed H=1536 rule; they are immutable binding metadata rather than additional mutable queue counters. The materialized root's normal_callbacks key stores Q. The runtime field last_upstream_generation is supplied by the Wash implementation identity SHA and its lifecycle meaning is confirmed by the Wash queue lifecycle contract.

The leaf does not define a pcm_ring_count field or a portable mapping from that name to queue counters. Queue field names and the exact runtime member list were supplied by Wash's implementation review and are used as a local canonical projection. Preserve each field independently. The only scheduler crosswalk is published_samples == F06W processed, established by the approved rule that the same valid span advances both; do not infer it from their equal cold zeros. No other counter is reconstructed from a similarly named field.

At each successful transition, run the actual F06W publisher and sample-block consumer once. Compare its valid output span with the queue append receipt and the resulting owner values: F06W processed and SampleBlockQueueState.published_samples each advance by valid; pcm_enqueued_samples advances by the samples actually appended; each complete 1024 block is consumed once; pending_sample_words contains all remaining actual F32 words in FIFO order. Check the corresponding count relationships at the same parent commit and abort the callback on disagreement. Do not derive a cursor, occupancy, or pending payload from the counters. The canonical Meter root key pcm_block_fill_cursor maps to the actual SampleBlockQueueState field block_fill_cursor:SampleBlockFillCursor. Its declared domain is U32; retain the typed value directly. The canonical spelling does not claim the runtime field has that name. If the owner has another payload field for pending samples, serialize that full field using its existing type/order.

## LFE delay ring

The standalone root contains the complete MeterLfeDelayState reported by implementation: source_samples, normal_callbacks, callback_index, samples_consumed, write_index, read_index and three optional DelayedLfeFrame values, each with origin and sample_words[1536]. This is one owner staged with every other meter leaf.

F06OF.3 defines the transition. Require g=samples_consumed//1536, advance write/read indices modulo3, write the current routed LFE input and generation g, determine output from the pre-advance samples_consumed value, then add1536 and advance callback_index. The first two outputs are newly owned positive-zero rows; from g=2 output origin is g−2. The rehashed frozen model spells corresponding fields model_samples_consumed, rows and input_frame_indexes. Their relation is bound by the same complete slot/generation transition, not by similar names. The row bytes correspond to sample_words; input_frame_indexes correspond to frame origin.

Normal LFE input uses the current semantically selected physical LFE row. Meter drains supply the fresh zero raw input row while the delay ring still advances. The full three-slot next state commits with the same parent root swap.

## Cold root and input-only labels

Each cold input-only case starts at callback0, input cursor0/read_index0, OAR/DV32/window/K/peak/integrated/ungated cold values, empty sample queue pending words and the three-null-slot LFE ring. The cases N=1, N=1537 and N=6,522,906 are synthetic metadata identities that exercise different schedule lengths. In particular, the N=1537 and selected-length snapshots are still cold pre-callback roots; their labels do not mean two callbacks have executed or that real audio was processed.

The materializer constructs the leaves from their individual cold contracts and portable bytes, assembles the root, canonicalizes it and hashes the resulting full byte string. It does not use the combined F06WH 13-owner hash or the 275-byte meter initializer/schema reference as a state seed. The sample owner member files contain the full canonical root bytes and can be hashed independently from MATERIALIZATION-RESULT.json.

For normal g<Q, source_cursor advances by actual A=min(1536,N−g*1536) and read_index increments once. For drains g=Q..Q+2 there is no source read: cursor=N and read_index=Q; callback clocks and all Meter leaves continue. The root and input cursor are one parent proposal/commit.

After the final drain callback has committed, capture the terminal Meter root bytes and compute meter_terminal_sha256 from those bytes. Then run the read-only final getters and make the separate GTC finalize commit with MeasurementReceipt. The receipt, finalized flag, and receipt digest stay outside the Meter preimage, so the hash is not self-referential. No real programme terminal hash was produced in this materialization task.

## Evidence boundary

Existing portable leaf bytes are reused for OAR, shared DV32, integrated and ungated state. True-peak FIR bytes and maxima follow the leaf's exact fields. The window/queue and LFE root layouts are deterministic local encodings because the approved leaves do not define complete standalone root serializers.

The exact SampleBlockQueueState field names came from Wash's implementation review and were not independently source-rehashed here. The approved F06W rules define the scheduler transitions and constraints used to check them. The LFE source transition is supported by F06OF and the hash-bound frozen model counterpart.

This package constructs input-only cold roots and scheduling/LFE boundary recipes. It does not run real Meter DSP, read PCM, compute a real MeasurementReceipt, or claim an actual terminal_meter_sha256. DRC remains outside this root.
