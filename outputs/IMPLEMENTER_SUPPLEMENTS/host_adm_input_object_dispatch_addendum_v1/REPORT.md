# ADM input-object dispatch addendum

## Static host roles

The host has an input-object execution wrapper that dispatches lifecycle calls to a selected reader object and an input-wrapper target arranger. The host WAVE input adapter also has a local Run-to-shared-source-time-metadata-normalizer call. That helper is shared across input adapters, so the local WAVE edge is not bound to the R138 `audio.adm` type.

The storage/parser library contains ADM/BWF file-storage components. No host callsite was recovered that transfers R138 `audio.adm` into a concrete ADM/BWF factory/provider. No first reader-output descriptor/metadata/frame handoff to the Atmos/codec consumer was recovered. The exact R138 reader factory and first Atmos/codec consumer remain `UNBOUND`.

## Smallest dynamic disambiguator

At the host input-object executor's virtual `Run` invocation, collect only the wrapper-owner token, selected reader object token, reader vtable/Run-target token, typed `audio.adm` input token under the same owner, parent task-generation token, thread identity, and caller stack. This identifies which reader object is invoked after selection. The input-type and generation owner joins remain open unless those tokens are captured from the same task. No payload memory is read.

No authorized audio sample was used in this static Ghidra turn; Wash/R4 is coordinating the single consolidated observation. No new native job was started.
