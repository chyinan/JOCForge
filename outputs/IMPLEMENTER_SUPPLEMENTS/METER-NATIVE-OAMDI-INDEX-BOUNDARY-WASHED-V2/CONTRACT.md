# Native table reader and bed/object updater index domains

## Bounded run-024 finding

In run 024, the runtime vtable getter selects the table-B row consumer and its append handler enters the native bed/object metadata updater. The same projected-record pointer is passed through the updater to the metadata formatter, and the same target-object pointer is later passed to the per-object state store.

The table reader and bed/object updater receive distinct index expressions: the table getter uses the absolute loop counter; the bed/object updater uses that counter minus a native start/base. Both happen to equal zero in the first observed record, but this observation does not establish a shared index domain or identity with F06OF target slot 0. The updater writes a 30-byte per-object state record at index 0.

The formatter's conditional scalar-conversion gate was zero in run 024, so the first-F32-to-compact-field branch was not taken. The projected record entered the formatter/state-store, but the captured run does not show the table-B scalar changing compact fields or EC-3 bytes. The job completed; only the output hash is recorded.

## Still open

Native bed/object index and F06OF target slot/generation remain separate and unjoined. The next output-relevant link is per-object OAMDI state to its OAMD/BSI serializer or queue, followed by any typed F06OF model adapter. Preserve `UNBOUND` for those links. No raw PCM, row payload, code address or encoded artifact was included.
