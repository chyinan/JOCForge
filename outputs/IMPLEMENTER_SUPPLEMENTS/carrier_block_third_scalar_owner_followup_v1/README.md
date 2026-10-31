# Third plan scalar owner follow-up

Status: the third U32 passed to the native transport planner is statically distinguished from `first_block`, but its first writer and semantic owner remain `UNBOUND`.

Ghidra-only review of the hash-bound Atmos program found the scalar read in the selected plan callers. The planner uses it for a first-segment capacity adjustment; the `first_block` assignment depends on the separate positive request-byte count. A direct-displacement survey found no store to the third scalar in those callers or elsewhere under the same frame-object displacement. Other instructions with the same numeric displacement use different base objects and were not joined. Bulk initialization or indirect writes remain possible.

Existing selected native plan-to-BSI owner/generation evidence is reused. The prior capture remains sample-specific and is not joined to any of the seven authorized BWF identities. No new native job was started.

The minimum next observation is one selected frame descriptor from construction through its first plan call. Watch only the third U32 field; record its object token, parent/frontend generation, initial value, first write site and writer input. At plan entry, record the three scalar/pointer arguments and the same owner token. Do not recapture the BSI presence writer or read payload bytes. Coordinate this observation only after the currently running authorized width-5 observation completes.
