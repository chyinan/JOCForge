# ADM time-field metadata survey for seven authorized BWF inputs

## Wash disposition

The upstream metadata-only package was independently hash-checked. It reports seven authorized ADM/BWF inputs inspected only for `fmt `, AXML, DBMD, and CHNA; all `data` chunks were skipped and PCM bytes read were zero.

## Bounded observations

- AXML contains `audioProgramme` start/end fields. The reported start is `01:00:10.00000` in all seven inputs; an end value is present.
- The survey found no named `offset`, `FFOA`, or `time_reference` field in the inspected AXML. Its DBMD/CHNA token scan also found no matching ASCII/UTF-16 markers. No BEXT or iXML chunk was present in the surveyed inputs.
- The metadata survey does not bind `audioProgramme` start to the DEE AtmosMezz `Start time` or `First frame of action` getter. It found no source field that explains `offset=auto` or `ffoa=auto` in these seven samples.

## Scope

This is negative evidence limited to the seven inspected samples and named metadata chunks. It does not prove that every ADM uses the same structure or identify a hidden binary field. Keep the auto-source edge `UNBOUND`; the separate default `file_position` conversion rule remains available and is not blocked by this survey. No DEE run, PCM/payload read, or key/license access occurred.
