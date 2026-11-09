# I0/D0 access-unit composition boundary

## Approved target contract

The 7.1 flat7x profile emits one ordered access unit with a distinct I0 frame followed by its D0 dependent frame: I0 at 640 kbps, then D0 at 1024 kbps. The 1664 kbps value is the aggregate AU rate, not one E-AC-3 frame. Build this as separate typed substream inputs in the isolated profile plan.

The target AU is publishable only as a complete unit from one source generation. Reject a partial I0-only result if D0 construction or validation fails. These are requirements from the Wash-approved profile contract; they are not proven runtime properties of the legacy reference.

## Architecture-only reference

A Wash-reviewed legacy reference demonstrated the abstract composition shape of a separate I0 frame input plus separate dependent-frame inputs/ownership plan, ordered I0 then dependent frame(s). This package carries only that high-level boundary. It contains no old source code, names, constants, source path, byte layout, rate values from legacy code, channel maps, or codec logic.

## Boundaries

`config_mode`, `phase_flag`, and the JOC reconstruction profile index are separate dimensions. ADM physical rows, E-AC-3 chanmap locations, and JOC carriers remain distinct domains. The D0 source-row/semantic-role join, native D0 audio algorithm, runtime source-generation receipt, and actual output remain `UNBOUND` / `NOT_RUN`; `production_allowed=false`.
