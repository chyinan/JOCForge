# Atmos DDP DRC control queue and candidate BSI boundary

## Added static rule

The Ghidra 12.2 package binds an internal 3-byte DRC control buffer. Its bit writer is MSB-first. Given the preceding optional-field inputs, the first DRC bit is at relative position:

`p = 2 + (record[0] == 3 ? 4 : 0) + 1 + (record[2] != 0 ? 16 : 0) + 1 + (record[4] == 0 ? 1 + (record[5] != 0 ? 4 : 0) : 4) + 1 + (record[8] != 0 ? 5 : 0)`

Relative to this internal control buffer, the presence flag is bit `p`, Line DRC is `p+1..p+3`, and RF DRC is `p+4..p+6`. The relative formula is not an absolute E-AC-3 AU/BSI offset. Profile-to-code values remain conditional on the previously established setup path; `film_light` maps to code 2 only if that path supplies the shown raw enum.

The 3-byte control buffer is sent through command ID 2 to an API queue; the queue copies the bytes into command-slot storage and marks it pending. A separate type-10/11 output path writes an output bit buffer and serializes metadata entries. The queued 3-byte payload has **not** been typed-joined to that metadata list. Do not claim it is present in the selected job's BSI or assign it a final AU bit position.

The statically followed control command and sample-array input are separate paths. This does not determine whether the selected decoder applies DRC to decoded PCM, nor whether the selected job activates the serializer.

## Scope and status

- Ghidra 12.2 static evidence only; no follow-up DEE run, output AU, PCM, payload, key, or license was inspected.
- The source package manifest, seal references, all 16 declared payloads, and all 133 declared source identities were independently rehashed in Wash. Every declared length/SHA matched; package checker was not run here.
- Queue payload → output BSI metadata entry, selected-job activation, and final AU bit offset remain `UNBOUND` / `NOT_RUN`.
- This supplement is additive to the existing DRC enum and control-boundary packages; it does not claim an end-to-end XML → selected job → BSI join or decoded-PCM effect.
