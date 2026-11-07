# Silence composite builder reference scope

## Bounded result

In the selected Ghidra 12.2 `SelectorProfile` program, the reference manager reports three references to `FUN_1801844c0`; one is a direct call at RVA `0x175e10` from the already-known `FUN_180175c80` path. The remaining two reference records were not categorized in this receipt.

No additional direct caller was found in this bounded reference set. This does not rule out indirect calls or uses outside the inspected program. The selected path remains a cleanup/forwarding branch; no global no-consumer claim follows.

## Status

No DEE runtime or media/PCM/payload/key/license access occurred. The potential silence-field consumer remains `UNBOUND/NOT_RUN`.
