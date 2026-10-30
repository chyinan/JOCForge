# Standalone output root → GTC parent binding

Result: existing approved definitions cover this interface. This crosswalk composes them without new native receipts, digest infrastructure, attempt syntax choices or output overwrite authority. All statements here are LOCAL_MODEL contracts; actual integrated implementation and production durability remain unverified.

## Authority and cold construction

The programme root owns the typed U64 AttemptId. A trusted parent-side private constructor/adapter takes that current AttemptId plus the existing programme-bound OutputOwner and provider configuration. It verifies that the OutputOwner/staging identity belongs to the same programme and attempt, then derives the legacy string as canonical ASCII unsigned decimal internally. Zero is "0"; no leading zeros, signs, whitespace, locale digits or alternate radix. It does not accept an independently chosen external attempt label as proof of binding.

The unique opaque programme staging_owner remains separate from the attempt number. Two programmes can both have attempt0. Never replace opaque ownership with the decimal string or infer it by parsing a filename. Existing staging/product naming and occupied-final-path policy are unchanged.

| Output cold/schema field | Binding and authority |
|---|---|
| profile/provider identity | Validate the existing selected provider, frame4096 and suppression2 against the validated programme plan; retain its exact provider SHA. OUTPUT-OWNER-ROOT committed state/provider; F15P.5. |
| attempt_id | Canonical rendering of that same parent AttemptId; not a second attempt or user field. OUTPUT-ATTEMPT-IDENTITY-BINDING producer/preflight/receipts. |
| staging identity/name | Existing unique opaque OutputOwner identity and its configured staging name under the existing namespace policy. GTC1/5 and keyless state staging_owner; binding filename_scope. |
| generation and next_source_index | Cold0. Output source calls advance once after parent commit; frontend callback0/1 without source do not advance this output source clock. OUTPUT-OWNER committed state/clocks; GTC3/4. |
| published count and durable length | Cold0/0; published length=4096*count using checked arithmetic. Suppressed sources do not increment either. Ordered AU ledger empty. F15P1/2/5. |
| finalized/product identity | finalized=false; retain configured product name, final product SHA absent. OUTPUT-OWNER committed serialization. |
| lifecycle | Existing GTC status owns METERING/READY/ACTIVE/RETIRED/FINALIZING/DONE. Staging ACTIVE unless abandoned; no new output-global lifecycle owner. keyless state/GTC1/6. |
| GTC output owner envelope | owner_id=output, current generation, actual canonical output state identity, constructor cold identity and owned state_ref. Digest does not replace the full root. GTC programme schema ownerState. |
| GTC output_ledger summary | Existing attempt_temp_id identifies this staging owner; OPEN and zero counts/bytes, last/final SHA absent. Summary must match authoritative output-root values; do not independently advance a second ledger. GTC schema outputLedger. |

Canonical OutputStateV1 bytes remain defined by OUTPUT-OWNER-ROOT. U16LE length-prefixed UTF-8 identity/name strings, raw32 provider SHA, U64LE counters, U32LE ledger count/entry layout and finalized/product identity are used exactly as specified, with checked representability. The new canonical decimal attempt supplies that existing string slot. A synthetic fixture's attempt "synthetic-0" or its fixed cold hash is not a production constructor input or production cold hash: actual owner/provider/names determine the new root's canonical identity.

The combined keyless-publication state schema also names timer/queues/codec, but these are not additional mutable fields owned by standalone output. Metadata and codec have their unique leaf owners; output consumes their staged receipts under the parent root. No secret/authentication state belongs here.

## Source proposal → durable evidence → parent commit

1. Preflight the current programme/typed attempt/opaque output owner/provider, source and output generation, before-state identity and expected source/dispatch/AU. Identity mismatch is rejected before IO or state mutation. String equality alone is insufficient.
2. prepare_source is pure over the committed before-root or its parent-staged clone. s0/1 propose next source/generation only, no append and no AU ledger entry. The upstream codec still builds/validates its complete4096-byte scratch; output suppression does not grant a codec skip.
3. s>=2 requires AU=s-2, expected_offset=AU*4096, exact frame4096, fixed SHA and CRC16 residual0. Emit the existing append proposal tied to attempt/staging owner/source generation/before state and proposed after-state/ledger entry. The root does not rewrite or repair codec bytes.
4. The trusted IO shell consumes that exact proposal once: check staging length/offset, explicit write loop, positive partial progress until4096, final length, readback equality/hash/CRC, flush/fsync. Only then emit DurableAppendReceipt bound to this proposal and same typed attempt/owner. Caller-supplied strings or durable=true flags do not constitute shell evidence.
5. The output leaf returns its proposed after-root and receipt to the GTC parent. The parent validates the same source/attempt/generation and all required leaf receipts, then swaps the entire staged root once. Output generation/source and logical AU ledger become committed together with metadata/codec and parent clocks. Do not append again or separately commit live output state before the parent swap.

In a standalone local unit test the leaf may expose a commit operation; integrated production must apply it only to the parent-staged root or install the proposed value at the single parent commit. Suppressed output progress is likewise a proposal until the parent commits. This is existing owner composition, not a new receipt framework.

## Finalization binding

EOF does not emit another frame. Compare source/AU/count/length and pending state with the current validated programme plan, not a fixed primary-song count. F02 gives Q=ceil(N/1536), sources=Q+2, published=Q, bytes=Q*4096; primary4249/4247/17395712 values are that fixture's plan. Metadata EOF can supply expected counts but is never a durable-output receipt.

The existing finalize proposal binds the same typed attempt/staging owner, committed output generation/state, product name, expected length and the existing binary durable-ledger digest. No pending append is allowed. Use the existing canonical ledger preimage; a JSON hash is not interchangeable. The trusted shell revalidates staging/full-product identity and performs required data fsync, existing finalization protocol, atomic namespace operation and parent-directory fsync. FinalExposureReceipt can be emitted only after the complete sequence succeeds and is tied to that proposal/attempt. Only then may the parent finalize transaction set finalized/product identity and DONE. Finalize advances no source or AU clock.

OUTPUT-OWNER-ROOT's finalize-intent is an existing test-only seam, not a new production recovery framework mandated by this crosswalk. Its presence is not a replacement for parent fsync or a durable exposure receipt. Reuse existing agreed output/error policy; do not add quarantine, auto recovery or overwrite behavior.

## Fatal, crash and restart

- Pre-IO rejection leaves output data/files untouched. Production error routing follows existing GTC classification; it does not silently adopt a caller label or relabel old staging bytes.
- IO failure after writes, readback/durability failure or append-success/root-swap-before-crash abandons the whole attempt. Committed numerical output root/ledger remains before the failed source; GTC status/fatal and abandoned control state record the failure. Cleanup truncate/delete is not same-attempt resume or filesystem rollback.
- Rename failing before namespace change has only its assumed atomic-failure guarantee. Rename success followed by failed/unsupported parent fsync can leave the final path visible but unconfirmed. No successful exposure receipt/DONE, no invented rollback of prior committed AUs and no promise the old product still exists after an independently permitted replace. Use existing DURABLE_OUTPUT_ERROR and finalize/parent-sync stage; do not automatically delete/re-rename/overwrite.
- full_restart is programme-origin only: close/abandon old staging, discard all old output receipts/proposals, all owners and measurement, checked attempt+1, new opaque staging identity/config and cold ledger, re-enter METERING. Do not reset only output counters or trust a surviving ledger as resume authority. U64 overflow is an error, never wraparound.

## Verified scope

This audit independently rehashed six approved base sources and both supplement manifests/payload identities. It checked four canonical attempt values, eight noncanonical labels and two owner distinctions with self-authored symbolic inputs. No integrated constructor/root implementation, new filesystem write, actual append/finalize, native execution or fresh programme publication was run.

OUTPUT-OWNER-ROOT remains READY_CANDIDATE_TEST_ONLY with production_allowed=false; its recorded local probe reported parent fsync unsupported, no final exposure receipt, product visible. These are bounded historical supplied test observations, not a new execution or production qualification. The selected-profile implementation contract is defined, while integrated runtime, filesystem/platform durability, power-loss recovery and product acceptance remain NOT_RUN or UNQUALIFIED as originally recorded.
