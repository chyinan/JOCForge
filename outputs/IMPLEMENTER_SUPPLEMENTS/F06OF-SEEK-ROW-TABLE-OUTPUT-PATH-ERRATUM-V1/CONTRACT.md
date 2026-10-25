# F06OF seek and row-table candidate classification

The latest static follow-up identifies an `AdmFileStorage::seek(samplePosition)` path and a `DlbWavFile` object with an internal per-channel row-pointer table. The traced conversion on that candidate reaches a stream-append writer, so this candidate is **not evidence** for the selected ADM input PCM row consumed by the meter.

No `ConverterChannelInputId` or selected source-owner identity is joined to that row table, and no same-generation connection reaches the selected AtmosProcessor measurement descriptor. The candidate row table therefore does not close:

`selected ConverterChannelInputId → physical PCM input row → selected meter descriptor → actual read/cursor/EOF`.

F06OF model lengths/cursors are samples per channel; Win32 file-read counts are bytes. The candidate's row-stride parameter has no verified unit in this path and must not be treated as a byte-to-sample conversion.

Status remains `UNBOUND`; no native run or PCM/row/payload inspection occurred. This erratum rules out only the traced output-append candidate as proof of the selected input row; it does not claim the entire binary lacks another reader path.
