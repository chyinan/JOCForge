# Wash review: ADM reader selection boundary

This package closes one static fact only: the current hash-bound DACT image contains an ADM BWF descriptor/parser and an `AdmFileStorage` path, including a read-method candidate. It does not prove the R138 `audio.adm` task selected that factory or returned that storage object. A numeric type display of 3 as ITU ADM/BWF is only a label mapping, not a typed join.

The `dee::input::AtmosMezz` / `audio.atmos_mezz` candidate is a separate class and cannot stand in for the R138 `audio.adm` reader. Existing R3 metadata receipts have no reader/vtable/Run identity, so they cannot identify it retrospectively.

The remaining minimum observation is one same-task-generation tuple at the host input executor immediately before the selected reader `Run`: owner, selected factory/module, returned pointer, reader object/vtable/Run target, and (if reached) nested DACT storage token plus first storage-method target. This is input-selection evidence only; it supplies no audio algorithm, formula, default, or field value.

Source Wash package was Ghidra 12.2 read-only/no-analysis. No IDA, native job, payload read, DLL, decompilation, raw address, or private key is included. Audio-affecting behavior remains DEE-only; external tools are limited to public syntax crosswalks or post-generation checks.
