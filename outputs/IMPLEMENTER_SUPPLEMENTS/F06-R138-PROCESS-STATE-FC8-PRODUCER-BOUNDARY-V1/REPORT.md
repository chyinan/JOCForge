# R138 process-state source correction and writer boundary

## Correction

The earlier `+0x7e4` source attribution was an indexing error. The record builder parameter is `longlong*`; `param_1[0x1f9]` therefore refers to byte offset `0x1f9 * 8 = +0xfc8`. The expression casts to `int`, and the process-state memory access is a 32-bit DWORD store/load, not a qword access at `+0xfc8`. The exact `+0x7e4` scan found two reads and zero writes; that offset is not the source of frame-record `+0x12c`.

## `+0xfc8` producer chain

The process-state constructor initializer first writes a DWORD zero at `+0xfc8`. Later, process initialization calls the source-channel configuration parser, which can write a DWORD derived from a parsed `channels` value to the same member. The record builder reads the low 32 bits and stores them into frame-record `+0x12c`; the previously established frame-record-to-receiver copy then carries those bytes to the component receiver.

The R138 job XML has no channel-named element, and the channels-provider owner has not been typed back to the R138 job decoder. Therefore the final R138 `+0xfc8` value and custom-BSI zero applicability remain `UNBOUND`. The constructor default does not prove the later runtime value.

## Scope

Incoming package `native-f06-r138-process-state-fc8-source-correction-addendum` manifest SHA-256 `c16759145be9eeb9337cf54373dbe096e1e7415bb2373e020c099a66d23d2a01` passed normal and `python -O` checkers. It corrects the prior `+0x7e4` claim and preserves previous scope-deviation disclosures. No DEE run, U16 capture, media, payload, or key read occurred.
