# Surround trim minus-three setter boundary

Reference-only Wash supplement for `custom_trims:surround_trim_5_1=-3`. The schema choice and a StringType setter path are bound, but the inspected callsite is the constructor's default `auto` value.

The setter writes a variable-length string to StringType value member `+0x08) and sets the initialized flag. The `-3` token has no numeric conversion or unit mapping in this evidence. A separate 16-bit consumer candidate reads a slot at `+0x968), but no same-owner link joins it to this StringType or the `-3) value.

Keep XML assignment, consumer identity, BSI, and audio effect `UNBOUND/NOT_RUN`. `IMPLEMENTATION_READY=NO`.
