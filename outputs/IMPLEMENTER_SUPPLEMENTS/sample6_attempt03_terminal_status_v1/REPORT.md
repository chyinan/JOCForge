# Human Is Cat attempt-03: terminal status

This is run-03 only; it does not reuse attempt-02's error text or static hypotheses. The implementation-side runner bound the authorized input SHA before launch and sampled the executable path's file hash while the PID was alive. It did not dump an in-memory image.

The same runner later wrote exit code 2. At terminal inspection, the target EC3 and same-name partial were absent. Stdout was zero bytes; stderr was 8,312 bytes. A narrow filter over run-scoped logs found no compare-point fields. The exact error text and expected/actual transaction values remain UNBOUND.

No output was parsed, decoded or played. No new attempt was started by Wash. The next useful step is one structured, run-scoped record immediately before the producer-token comparison; without it, this receipt proves only the process/input identity and exit status.
