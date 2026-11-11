# Flat7x D0 block-2 skipfld structure

## Approved target

This supplement supports one structure-only increment for the approved 1664 kbps target: I0 640 kbps followed by D0 1024 kbps, D0 chanmap 0x1A00, flat7x/JOC index 1, 15 objects/rows, and one complete EMDF container in D0 audio block 2. The field's U9 byte count C must be 1..511; an overflow fails closed and never continues into a neighboring field.

## Field units and parser rule

The E-AC-3 syntax writes a one-bit skiple flag, a nine-bit unsigned skipl value when present, then exactly skipl bytes of skipfld data. skipl counts data bytes only; it does not include the flag or the nine-bit length field.

For EMDF, the 16-bit container length gives body bytes after the four-byte sync prefix. Define C_total = 4 + body_bytes. OpenJOC classifies an EMDF carrier as Parsed only when bytes_consumed equals the entire supplied skipfld slice. For the observed official D0 block-2 fields, every complete field therefore has C_total = skipl. No adjacent field concatenation or partial-container acceptance is part of this contract.

The complete wire span is 1 + 9 + 8*skipl bits from the skiple flag, or 9 + 8*skipl bits from the start of the U9 skipl field.

## Official reference observations

The user-designated official MKA has 23,033 access units. Each is I0 LegacyIndependent/BSID6/2560 bytes followed by D0 Dependent/BSID16/2560 bytes. D0 chanmap is 0x1A00. Every D0 block-2 skipfld contains one complete EMDF/JOC container; OpenJOC accepted ID14 headers with downmix index 1, channel_count 7, and 15 objects. Observed D0 block-2 C_total/skipl lengths range from 301 to 377 bytes.

The official file's rate is I0 640 + D0 640 = 1280 kbps. The approved target is I0 640 + D0 1024 = 1664 kbps. Keep those rate allocations separate. For the target, the I0 frame extent is 2560 bytes and D0 extent is 4096 bytes per 32 ms access unit.

## Bounded implementation use

An additive pure writer may accept caller-supplied opaque bytes for one EMDF skipfld, preflight the complete bit capacity, write skiple=1, write U9 skipl equal to the supplied byte count, then copy those bytes once in order. It rejects empty/non-field input, any count above 511, arithmetic overflow, and insufficient capacity before modifying the destination. It never fragments a container or crosses into block 3.

This helper contract covers skipfld syntax and fail-closed assembly only. It does not generate EMDF/audio/JOC payloads, choose the block-2 insertion point, map ADM source rows, or establish source owner/generation. Exact OpenJOC acceptance of a complete 511-byte EMDF remains NOT_RUN; the 511-byte policy fit vector is arithmetic-only. Full Flat7x production remains disabled.
