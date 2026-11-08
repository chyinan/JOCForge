# Explicit offset/FFOA conversion boundary

Both explicit XML properties statically enter FUN_1401359e0 with timecode_frame_rate and return floating seconds coordinates. The decimal branch parses h/m/s/fraction; the frame branch parses integer or colon-frame forms, with a static lowercase df path. This is static converter behavior, not proof that XML admission succeeds. Strict token widths/whitespace/trailing-character rules and the full frame-rate vocabulary remain UNBOUND.

The auto normalizer separately resolves offset and rebases/clamps FFOA; the first DDP consumer reads both values. Source metadata field/chunk identity remains open: the ADM time token reaches descriptor+0x60 and master+0xa8, but exact XML origin is unbound; FFOA reaches descriptor+0x58/master+0xb0 without a bound source writer. Existing six N36 and one N38 inputs have no distinct nonzero offset/FFOA candidates in the scanned metadata. No DEE or atmos_info run occurred.
