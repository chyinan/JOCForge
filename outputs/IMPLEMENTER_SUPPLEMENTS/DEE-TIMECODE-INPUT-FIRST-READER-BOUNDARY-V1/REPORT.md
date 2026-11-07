# Input timecode_frame_rate first-reader boundary

Wash correction to the earlier timecode field note: the setter's `param_1` is `longlong *`, so `param_1+0x31` is QWord-indexed pointer arithmetic and refers to byte offset `+0x188`. This supersedes the earlier ambiguous offset wording only; it does not close the input owner.

The host setter `FUN_1400f13e0` parses the exact `timecode_frame_rate` key and passes the value to `FUN_14001d440` at byte `+0x188`. A bounded input-side reader candidate `FUN_1400f09d0` reads its own object `+0x188`, copies the value to a local string, and passes it with a local offset to `FUN_14012ff20`. The reader carries an `AtmosMezz::Run` label, but there is no typed owner/caller join between the setter object and reader object. Therefore the candidate is not proof that `/input/audio/atmos_mezz` or `/input/audio/adm` field reaches it.

The converter body and downstream effect remain uninspected/not run. DDP output timecode path is separate and unchanged.