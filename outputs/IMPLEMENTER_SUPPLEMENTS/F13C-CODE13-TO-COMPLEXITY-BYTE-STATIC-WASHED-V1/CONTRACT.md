# Parameter code 13 to selected complexity byte

## Wash result

Ghidra evidence for one hash-identified Atmos DLL closes a static same-worker scalar path:

1. The selected config builder reads parameter-list entry code 13 into an intermediate config value. If the entry is absent, the static default is 11. A supplied scalar is copied only subject to a separate validation call whose runtime acceptance is NOT_OBSERVED.
2. Parameter code 14 has a separate value and default of 0. A positive value controls a separate gate. A type-4 entry from another parameter list feeds a third worker field.
3. The selected writer traverses a child-worker table and places the low byte of the code-13 config value into the selected worker's complexity byte. The selected consumer uses the same static child-table path to pass that worker's source record to the import routine.
4. The referenced sealed importer evidence maps that record byte to working-header byte 119; the referenced sealed F12 field map labels byte 119 as the 8-bit complexity_index_type_a field at bits 105-112.

This establishes a static native producer path from parameter code 13 to that selected F12 field. It does not establish that a user-facing XML/task field supplies code 13, that a runtime instance takes the selected branch, or that a semantic ID14 object-count producer owns the same byte.

## Separate values and limits

Code 14 and type-4 feed separate worker fields; numeric labels do not join them to a semantic ID14 object count. The conditional scalar vectors are static expectations only. They do not show that a real host accepts the supplied values.

Runtime object identity, parent/source generation, host XML to code-13 entry, semantic ID14 object-count to this byte, carrier-block owner, and runtime call order remain UNBOUND or NOT_OBSERVED. The native pair is NOT_RUN. No PCM, encoded payload, or key material is included. This evidence is for mapping only and does not authorize algorithm or readiness changes.
