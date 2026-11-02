# AtmosInfo master time-field writer boundary

## Wash disposition

The upstream Ghidra 12.2 static package closes the writer of AtmosInfo's Start time and First frame of action output fields to a BWF reader interface. It does not resolve the concrete reader implementation or map those reader outputs to specific WAVE chunks/ADM metadata fields.

## Typed writer path

- The BWF registration path installs an `AdmFileStorage` factory callback. The factory constructs an `atmos_storage_master_s` object.
- The master open method `FUN_1402598d0` obtains a per-input reader through the factory object stored on the master. It queries reader virtual slots `+0x28` and `+0x38`.
- The `+0x38` value is written to master `+0xA8` (Start time). The `+0x28` value is treated as an FFOA candidate; master `+0xB0` is derived from both values through an FFOA-validity branch.
- The reporting accessor reads the master fields and formats them as Start time / First frame of action.

## Remaining source edge

The factory vtable `+0x08` target and reader getter methods `+0x28/+0x38` are unresolved in the available `atmos_info.exe` static image. A separate generic RIFF/RF64 reader is known, but no call/dataflow edge connects it to this master reader interface. The seven-sample metadata survey found no named offset/FFOA/time-reference field and did not bind AXML `audioProgramme` start to the master fields. The concrete source chunk/field remains `UNBOUND`.

This receipt does not provide an auto-offset or FFOA input rule. No DEE/atmos_info run or PCM/payload/key/license access occurred.
