# F13C writer-success serialization boundary

This supplement records the static boundary between the JOC parent candidate and successful writer state. It explains why the current capture plan cannot obtain a native writer-success receipt while also stopping before serialization and output creation.

## Static success ordering

The parent success counter advances only after the parent writer returns success. The parent writer invokes the bit writer first. The bit writer validates its buffer/header/sequence inputs, writes the 8-bit header and 10-bit sequence, then writes the remaining fields and payload bits by mutating the output buffer/cursor. Only after this serialized write path returns success does the parent update its counter (including the 1023-to-1 transition).

Consequently, stopping before the first bit-buffer write can capture callback entry and candidate inputs, but cannot observe native writer status or the success-counter-after value. A stub or synthetic success return is not native writer evidence.

## Output-sink decision

- A file sink can create encoded bytes before cleanup; cleanup afterwards is not a no-output guarantee.
- An in-memory sink could avoid a file artifact, but it still executes protected bit serialization. It does not satisfy a stricter requirement to stop before serialized output exists.
- The existing run_011 harness is measurement-only and has no JOC parent-writer stop hook. It is not reusable as a JOC writer-success receipt.

Wash decision for the current scope: keep the same-row/generation and writer-success join `UNBOUND`; do not start a native run. A later controlled-sink capture that permits serialization requires a separate explicit Wash GO and must have an independently verified no-file boundary. A pre-serialization-only capture may bind inputs but must keep writer result and counter-after as `NOT_OBSERVED`.
