# D8 store-domain separation and getter owner

## Bound getter owner and initial state

For the selected AtmosProcessor path, the getter receiver is its inner state stored on the outer object. The inner-state constructor zeros the relevant +0xD8/+0xE0 qwords, and one selected update routine writes neighboring fields but not +0xD8. This bounds the getter owner and initial state for that constructor/update path; later writers and other routes are not excluded.

## Distinct +0xD8 writer domains

Hash-bound Ghidra evidence now distinguishes several objects that contain a +0xD8 field:

- The selected AtmosProcessor inner state read by the PcmMd getter.
- A separate allocated PCM metadata record whose +0xD8..+0xE7 pair is written and queued within a reader worker; the same worker queue consumer pops that record and passes it to a serializer view.
- A separate CBIReader inner object with its own outer type and allocation; a branch-local writer copies adjacent values and clears a byte at +0xD8.
- A separate 0x100-stride processing-row layout with its own owner/value source.

The CBIReader and queued-record paths have not been joined to the AtmosProcessor inner state. No explicit pointer transfer, same-object identity, semantic field name, or generation token establishes that any of these stores supplies the selected PcmMd getter's +0x28 field. Matching +0xD8 offsets are not identity evidence.

## Remaining F06OF boundary

The selected builder still has an empty OAMDI event list; this is limited to that builder. Other producers/modes are not ruled out. No native capture or generation token is present. F06OF target slot and generation remain UNBOUND. No PCM, EC3 payload, private key, or raw instruction address is included. No codec algorithm or production-readiness conclusion follows.
