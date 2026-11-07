# F06ND generic text-event producer boundary

One generic text-event path parses a text line, sends a parsed event to a callback/local collector, then submits the finalized collector through a keyed entry. The callback uses the same first per-event consumer as a separate typed storage-event path, but a shared code target does not prove the two event owners or generations are the same.

The selected Category5 source remains unjoined to this parser/callback/collector (`NO_SELECTED_JOIN_ESTABLISHED`). DRC/BSI effect is `UNBOUND/NO`, runtime is `NOT_RUN`. This is a scoped result, not a global no-event claim; no event-record layout was reconstructed.

No source, decompilation, binary, PCM, payload, key, authentication material, local absolute path, or raw native address is included.
