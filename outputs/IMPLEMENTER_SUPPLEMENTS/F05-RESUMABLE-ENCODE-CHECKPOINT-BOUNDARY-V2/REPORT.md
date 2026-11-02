# F05 transaction and restart-recovery boundary

**Status:** current selected source paths agree across the audited and current HEADs; historical failure operands remain unbound; no persistent per-source resume checkpoint exists in the selected production path.

## Producer-token path

The first compare is `ID14 provenance producer_transaction_id == Some(source_transaction_id)`. The active CLI creates `jocforge-source-{source_index}` and separately passes `jocforge-main-callback-g{callback_index}` as the parent GTC receipt token. The current explicit source-execution path keeps those roles separate. A convenience wrapper that copies the source token into the GTC token has no callsite in the selected CLI/application path.

Historical attempt 02 recorded `Joc(ProducerTransactionMismatch)` but lacks executable identity and compare operands. Attempt 03 lacks exact error text and compare-point fields. Neither failure is tied to the current build. Attempts 07 and 08 are distinct callback-index and post-rename fsync edges. Do not change current token wiring by guessing which historical operand was wrong.

## Restart boundary

The source path commits `ProgrammeOwnerRoots` in process memory. AU staging verifies offset/readback/CRC, flushes and fsyncs the attempt file, and returns a durable append receipt before `commit_source_advance` returns the next owner roots. On failure, staging is abandoned and the file retained; no process-restart source checkpoint, root restore, or replay/reopen path exists in the selected production modules. EOF finalize-intent is terminal publication evidence, not an intermediate checkpoint.

The refreshed Wash contract makes the minimum resumability boundary concrete: persist a restorable 12-owner root snapshot and generations together with run/attempt/input/config/executable identities, callback/source/dispatch/AU position, exact durable append offset, prefix hash, CRC/fsync receipt, and last frame hash. Write it atomically only after append and owner validation. On restart, verify the committed prefix; truncate/fsync only a valid uncheckpointed tail before replay. If identity, prefix, or root state cannot be restored, retire the attempt and restart from the input. Resolve EOF `ExposureUncertain` separately.

The included synthetic fault vector models a crash after AU1 is fsynced but before checkpoint 10 replaces checkpoint 9. It is **NOT_RUN**. This is a long-run reliability item, not an audio-algorithm mismatch; it must not block the current matched PCM comparison or first playable MVP.

## Source and execution scope

The source package manifest and four payload hashes passed Wash verification. Its source review was refreshed to commit `b93b9a34016c0cd669de1e82a3f2ee5466518b02`; the 15 selected files show no Git content diff from the earlier `a18b429…` audit baseline. No package seal file is present. No code change, encoder run, test, media/PCM/payload read, or key/license access occurred in this Wash pass.
