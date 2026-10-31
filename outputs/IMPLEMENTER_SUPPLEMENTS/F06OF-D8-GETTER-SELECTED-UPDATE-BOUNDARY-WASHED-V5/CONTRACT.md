# AtmosProcessor getter and selected-update boundary

## Bounded getter path

For the hash-identified Atmos DDP module, a DDPJocEnc state stores an AtmosProcessor outer pointer. The selected getter path reads the inner-state pointer from outer +0x8 and copies inner +0xD8/+0xE0 into the PcmMd output wrapper fields +0x28/+0x30.

The outer constructor creates a 0x1A8-byte inner state. Its initializer zeros ten qwords including +0xD8/+0xE0, so the selected getter source slots begin at zero for this construction path.

## Selected process update

One selected process-update entry reaches an update routine on the same AtmosProcessor inner receiver. That routine writes inner +0xB0, +0xB8, +0xC0, +0xC8, +0xD0, and +0xE0, but not +0xD8. This is bounded to that function and branch; it does not exclude other writers or routes.

## Separate D8 writer domains

The queued PCM record writer/consumer path, CBIReader inner-object writer, and 0x100-stride processing-row writer are separate object/layout domains. The inspected graph has no same-pointer transfer from any of these domains to the selected AtmosProcessor inner getter and no common generation token. Equal offsets do not join fields.

## Open boundary

The selected getter owner and constructor default are statically bounded; the later writer and semantic name for inner +0xD8, same-generation token, native capture, and F06OF target bridge remain UNBOUND/NOT_CAPTURED. Other routes are not excluded. No codec algorithm, production-readiness, or global absence claim follows.
