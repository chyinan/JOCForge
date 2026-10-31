# Separate +0xD8 record writer and queue boundary

## Washed result

Hash-bound Ghidra evidence identifies a bounded writer/queue path for a separate PCM metadata record. The producer allocates and initializes a record, copies a 16-byte pair from provider data into record-relative bytes +0xD8 through +0xE7, and transfers that record pointer into a ring/queue. The provider is reached through an interface accessor, but the meaning of its selected source fields and any generation token are not identified.

The selected PcmMd getter is a different call path: it reads its own receiver-relative +0xD8/+0xE0 into a wrapper field. In the inspected static graph, the queued record pointer is not transferred to that getter receiver, and no shared call or generation identity joins them. An equal offset does not establish a shared owner or field. A separate processing-row store at the same displacement is another domain and is not joined either.

This closes only the existence and owner/queue boundary of the candidate record writer. It does not close the first writer of PcmMd's +0x28 wrapper, the relation between this candidate record and that wrapper, or any F06OF target identity.

## Remaining boundaries

The selected builder still reports OAMDI event count zero and event pointer null; that conclusion is limited to the selected builder. Other producers/modes are not excluded. Native capture and runtime generation are NOT_RUN/NOT_CAPTURED. The DDP workflow did not run because setup failed before workflow execution; this is not a no-hit.

The package also contains converter-side ADM metadata evidence, but that is a separate input-side path. Do not merge its ChannelInputId, host slot, descriptor U8/U16, OAMDI event U16, OAR event U16, global target ID, or TargetSlotIndex based on equal values or offsets.

No PCM, EC3 payload, raw disassembly, or private key is included. No codec algorithm or production-readiness conclusion follows.
