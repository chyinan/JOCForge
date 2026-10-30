# Native OAMDI 48-byte property aggregate boundary

## Bounded native finding

Static analysis of the selected native per-object update loop shows a 48-byte property record copied into a native aggregate buffer after the state update. This establishes a native buffer-layout/copy edge only. A 48-byte extent does not identify or equal the 52-byte table-B/control52 row.

In run 026, a read watch on the first four bytes of the per-object state destination stopped immediately after the initial 16-byte state-store write. No downstream read was observed. The OAMDI-to-EVO functions were not hit in the selected EC-3 job. Therefore the property aggregate's consumer, owner/generation, and join to the final EC-3/BSI serializer or queue remain open.

This does not establish an F06OF row/word identity, a native callback generation, or that the run-024 table-B scalar changed stored state or EC-3 bytes. Preserve the native bed/object index, F06OF target-slot index, control16 row, and model generation as separate domains until a typed join is observed.
