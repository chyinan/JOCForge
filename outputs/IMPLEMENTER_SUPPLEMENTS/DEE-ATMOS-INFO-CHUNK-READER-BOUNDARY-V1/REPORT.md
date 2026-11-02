# DEE AtmosInfo output fields versus generic WAVE reader boundary

## Wash disposition

The upstream static trace binds the values formatted as AtmosMezz `Start time` and `First frame of action` to two fields in a master record. A separate generic RIFF/RF64 reader is known to parse WAVE containers and data chunks, but this receipt does not bind a specific chunk reader's field to either master-record field.

## Bounded facts

- `FUN_14006c840` reads master-record `+0x50` as Start time and `+0x58` as First frame of action; `FUN_14006dbf0` formats the outputs.
- The generic `FUN_140214430` reader recognizes RIFF/RF64/WAVE, `ds64`, `fmt ` and `data`, with a `JUNK` special case.
- The writer/initializer for master `+0x50/+0x58` and its join to a concrete metadata chunk or exact WAVE reader output remain `UNBOUND`.
- The companion seven-sample metadata survey found no named offset/FFOA/time-reference fields in inspected AXML, DBMD, or CHNA; it did not bind `audioProgramme` start to the AtmosInfo outputs.

## Scope

This narrows the next DEE edge to the master-record field writer and any reader call that populates it. It does not supply a source rule for `offset=auto` or `ffoa=auto`; the default `file_position` MVP is not blocked. No executable run or data/PCM/payload/key read occurred.
