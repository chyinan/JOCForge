# ADM track count and native admission boundary

**Status:** 36/38 rows pass the observed Atmos-plugin row-count gate; the core `Comp` frame-count source and active capacity remain unbound. The 36-row run reached the ID14 writer entry but did not execute the writer instruction; a complete 36-row E-AC-3/JOC output is not established.

## Evidence across the two authorized inputs

Metadata receipts bind My Jealousy to 36 channels, `block_align=108`, 36 CHNA/DBMD rows, 10 beds and 26 objects; the 人是猫 receipt binds 38 channels, `block_align=114`, 38 CHNA/DBMD rows, 10 beds and 28 objects. These are distinct input identities; the metadata count alone does not identify the native `AtmosReader` format descriptor or core frame view.

The bounded Ghidra path separates several counts:

- `AtmosReader` reads a U16 channel count at descriptor `+0x0A`, rejects only zero, and copies it to packet `+0x08`. Packet sample count at `+0x10` is a separate field; its gate accepts 1,536 or zero/flush.
- The Atmos plugin copies six-byte packet rows. `FUN_1801AC670` accepts row counts 1–128. Prior authorized observations saw 36 and 38 inner rows; the 36-row path reached the ID14 writer entry, while that writer instruction did not execute. For the 38-row receipt the writer input words were `[3,15,0]`; its instruction likewise did not execute.
- `code13-code14` is the separate ID14 object count, not the physical ADM track count. `FUN_1802321a0`'s per-object loop is another separate count domain.
- Core `FUN_140213470` reads frame-view count `N` at `+0x20` and a pointer list at `+0x08`; it stages `N` pointers in a 16-qword local array before the later capacity check. `FUN_140315db0` checks descriptor-group count against `root+0xC138`, then checks a per-group count against `root+0xC140`, returning `0x1D` when the count exceeds that selected capacity. Neither the active capacity nor a typed ADM-row → core-frame-count edge is known.

## Provisional implementation scope

This evidence supports removing a fixed **38-row assumption at the plugin packet-row materialization boundary** and representing the observed row count dynamically, with a 1–128 bound at that boundary. It supports trying the authorized 36-row input as the next MVP validation target alongside the existing 38-row path.

It does **not** support equating CHNA/physical tracks, Atmos packet rows, `code13-code14`, and core frame-view count. Keep those fields distinct. Do not use the 1–128 plugin bound as the core buffer capacity; the core frame view has a separate 16-entry local staging array and a later selected capacity whose value is not observed. Full 36-track encode remains unproven until JOCForge completes and validates the actual output.

## Scope and limits

Wash verified this source package's six listed payload hashes, manifest, sidecar and seal; the source checker passed normally and under `python -O`. This slice ran no DEE, read no PCM/payload/key/license, and contacted no implementation agent before Wash review. The 38-row output receipt is not a local rehash of the output payload. The paired pointer/count observation for the 36/38 routes is only a proposal and is **NOT_RUN**.
