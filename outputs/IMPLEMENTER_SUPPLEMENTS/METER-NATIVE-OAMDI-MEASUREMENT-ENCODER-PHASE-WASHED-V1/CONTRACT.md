# Measurement-only OAMDI state and separate encoder pass

## Wash phase boundary

The run-032 state scalar edge is measurement-phase evidence only: same OAMDI state owner `S`, field `+0x18` signed I32 → helper → DSP fifth argument → DSP context store, joined by static field flow and a direct call stack. The field's semantic name remains `UNBOUND`.

The encoder pass is a separate phase. In run-033, the encoder marker and encoded-block wrapper were reached, while the watched state/DSP functions did not hit before successful exit. In run-034, the encoder block wrapper reached the FileWriter entry for frame index 0 with a 4096-byte request; the static path uses `fwrite`, the buffer was not dereferenced, and the job completed. These observations show encoder/FileWriter activity but no state-scalar-to-encoder or state-scalar-to-output join.

No OAMDI converter-to-EVO function was hit in the selected EC-3 job. The FileWriter stream index and its exact FILE-to-named-output-path identity were not captured here. A separate host/file-sink reference package covers another run but is not a typed cross-run join. The identical output hashes do not establish that the OAMDI scalar affected output.

## Limits

Native callback generation, the OAMDI field's semantic name, F06OF target/control/generation adapter, DSP/property output to final EC-3/BSI sink, and whole-chain output remain unbound or unrun. No PCM, metadata row payload, control payload, encoded buffer or output bytes are included.
