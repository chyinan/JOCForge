# Native OAMDI output-consumer evidence

## Scope

This supplement records a bounded native DEE observation for the selected ADM/profile path. It does not describe JOCForge runtime behavior and does not establish a global absence of OAMDI-to-encoder connections.

## Bound output path

Static tracing and run-036 observations bind the selected encoder output list to one FileWriter stream. The configured output path matched the run job's EC3 path. The FileWriter object remained the same through open and write; the returned `FILE*` matched the `fwrite` argument. Frame 0 selected stream 0 and requested a 4,096-byte write. The native output completed.

This package does not include output bytes or PCM. It preserves the output hash only as a source identity value; equal hashes across runs are not a pointer, generation, or owner join.

## OAMDI phase boundary

The captured per-object state `+0x18` scalar reached a DSP context during the measurement pass. Its semantic field name and native generation remain unknown. In the selected encoder passes for runs 035 and 036, the monitored 48-byte aggregate copy and DSP consumer did not hit through the FileWriter call and successful job completion.

That no-hit is bounded to those runs and monitored functions. It does not prove that every OAMDI property or alternate path is absent from encoding.

## Open lineage

The native bed/object index is not joined to F06OF target slot. The table-B property is not joined to a F06OF control16 word. Native callback/object index is not joined to model generation. The F06OF slot/word/generation crosswalk remains `UNBOUND`; product policy is unchanged.
