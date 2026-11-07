# DEE XML time-control boundary

## Disposition

Reference-only Wash evidence. `start/end` and input `offset/ffoa` are separate fields and time domains. Neither path is closed from selected XML input through a typed runtime owner to the Reader/sample seek.

## start/end

The schema treats `encode_to_atmos_ddp/start` and `/end` as optional strings with document defaults `first_frame_of_action` and `end_of_file`. The sanitized R138 input-only vector contains `start=86640`, `end=86664`; it is not a runtime observation. A reused static file-position/embedded-timecode range-record-to-Reader path exists, but no typed join connects these XML leaves or their omission state to that range record, Reader cursor, or total.

## input offset/ffoa

The input `adm/offset` and `adm/ffoa` leaves are separate optional strings with schema default `auto`. One static host edge is bound: `AtmosMezz::Run` calls the normalizer at RVA `0xf10da`, passing FFOA policy from `+0x148` and offset policy from `+0x168`; that edge writes a host-local property list / `AtmosMezz+0x130`. Separately, `FUN_18007fba0` consumes map keys `offset` and `first_frame_of_action` from `PrivContext+0xa8`. The XML-to-policy-owner join and host-response-to-converter context join remain `UNBOUND`; Reader seek/sample effect is not captured.

Do not merge `start/end`, `time_base`, `offset`, and `ffoa` into one timeline rule. No units, effective runtime values, sample positions, Reader seek, or audio effect are claimed.
