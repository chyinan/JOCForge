# F13C selected BSI producer correction

This correction supersedes Section 1, “JOC complexity producer,” in the base package file `specs/13-joc-complexity-timer-payload-order.md` for the selected additional-BSI scalar path. The base file still exists for provenance, but its Section 1 must not be implemented as written.

The earlier claim that the producer parses the serialized EMDF, requires ID14/ID11, and derives the selected additional-BSI byte from an ID14 object count is withdrawn. The newly observed selected path obtains an I32 from the configuration record before EMDF serialization, then writes the configured count into the additional-BSI record. The selected result happens to be 16; that numeric equality does not validate the earlier ID14 derivation.

Do not infer an ID14 semantic parser, common ID11/ID14 identity, carrier-block origin, or encoded malformed-input behavior from this package. Those remain separate unresolved work. `F13C` and overall `IMPLEMENTATION_READY` remain `NO` until those dependencies are washed and validated.
