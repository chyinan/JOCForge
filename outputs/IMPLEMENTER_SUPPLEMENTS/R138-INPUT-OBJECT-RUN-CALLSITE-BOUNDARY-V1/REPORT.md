# R138 `InputObject::Run` static boundary

## Finding

For the hash-identified host image recorded in the source package, Ghidra places the candidate reader load, vtable load, and virtual `Run` call at `dee.exe+0xCA9F2`, `dee.exe+0xCA9F9`, and `dee.exe+0xCA9FC`, respectively. The last address is a useful stop point immediately before the virtual call.

This is a static callsite boundary only. The available dynamic activation was `dee.exe --schema` without a task profile or audio input. It selected no route and ran no encode. It did not invoke or identify the R138-selected reader.

## What remains open

The `audio.adm` owner-to-selected-reader/factory join, field writer/constructor identity, factory table/count/slot, and parent generation remain `UNBOUND`. A scan found 256 displacement candidates for `+0x1F0`; matching displacement values do not prove object identity. The schema metadata callbacks are not R138 reader selection.

## Use of this supplement

Reference only: use the callsite as a bounded observation point for an already-authorized R138 activation. Capture pointer/scalar metadata before `Run`; if any owner, factory, writer, or generation edge is not observed in the same activation, keep it `UNBOUND`. Stop before audio processing. This package supplies no audio algorithm rule and does not authorize a new encode or change implementation behavior.

No new encode was started for this investigation. No R138 XML, row/table bytes, PCM, payload, encoded output, key, or license content was read. The observation plan remains `NOT_RUN`.
