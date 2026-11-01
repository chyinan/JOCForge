# R138 reuse boundary for AtmosMezz provider capture

## Result

`R138_ROUTE_MATCHES_BUT_INPUT_IS_ADM_NOT_ATMOSMEZZ; PROVIDER_PACKET_EDGE_NOT_REACHED_IN_R138; NEW_NATIVE_CAPTURE_NOT_RUN`.

The hash-bound R138 execution XML selects `encode_to_atmos_ddp`, but its input is `audio.adm` (`selected_adm_38ch.wav`), not the `atmos_mezz` reader path. R138 therefore cannot supply an observation of the AtmosMezz storage-provider member or its Run/GetTargets callbacks. Wash confirms R138 is complete and has no live PID or WinDbg session, so it cannot be attached to for a new stop.

The prior R138 reader/provider-to-codec and codec-to-writer observations do not identify the AtmosMezz provider object or the plugin process packet used by that reader path. The existing R138 run cannot close the requested provider-to-process same-object/generation edge. The active R4 run08/09 is a different task/thread and remains unavailable for reuse until Wash verifies route/input identity and absence of output conflict.

## Status and next action

For R138, the AtmosMezz provider boundary is `NOT_REACHED`; for the broader host provider-to-plugin packet relationship, same-object identity is `UNBOUND` and parent/source generation is `NOT_OBSERVED`. Do not start a duplicate job. If Wash confirms R4 is a compatible task-owned run, use a one-shot scalar-only observation on that same activation; otherwise have Wash coordinate any new run. Capture only reader/provider token, process packet token, route/factory index, and parent/task-generation. Do not read or save PCM or bitstream data.
