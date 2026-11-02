# ADM auto-time host property consumer boundary

## Bound facts

Ghidra 12.2 static evidence binds the host path `AtmosMezz::Run` → normalizer → `AtmosMezz+0x130` property rows. The typed `AtmosMezz` vtable slot `+0x10` (`FUN_1400efd60`) reads those rows when answering `dee_audio_input_target` and serializes them into a host input-property response.

The separate converter image reads `offset` and `first_frame_of_action` from its separately allocated `PrivContext+0xa8`. The selected host consumer has no converter-object argument and no typed transfer to that map was found on this bounded path.

## Implementation consequence

`file_position` remains a viable default-position MVP path. This receipt does not close `offset=auto` / `ffoa=auto`: their host response-to-converter adapter and same-input owner/generation remain unbound. Do not guess that equal keys, values, or nearby objects imply a transfer.

R3 is continuing the next adapter-edge investigation. This package reports a narrow static boundary, not proof that no other adapter exists and not a PCM/audio difference result.

## Source receipt

Wash recomputed the package manifest and seal hashes, ran `CHECK.py` normally and with `python -O`, and reviewed the scoped payloads. The source package records `DEE_EXECUTION=NOT_RUN` and no PCM/media/payload/key/license access.
