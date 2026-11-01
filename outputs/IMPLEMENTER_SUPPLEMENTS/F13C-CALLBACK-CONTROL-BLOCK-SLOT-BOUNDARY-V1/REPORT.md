# F13C callback control-block slot boundary

## Result

The hash-bound Ghidra project resolves slot `+0` for a newly constructed `std::_Ref_count<AtmosMasterFileStorageDesc>` control block. It does not identify the table at the older nonzero receiver used by the F13C dispatch site, nor does it reach a preferred/downmix field or DDP-JOC/EC3 writer.

## Closed static edge

`FUN_1403f1a50` allocates an 0x18-byte control block, installs vptr `0x140bcf970`, and stores the produced callback object at `+0x10`. RTTI names the control-block template `std::_Ref_count<AtmosMasterFileStorageDesc>`. Its slot `+0` points to VA `0x1403f2a40`, RVA `0x3f2a40`; the first branch reads the object pointer at control-block `+0x10` and tests it for null.

The two identified initializer callers clear both pair fields before constructing this control block. They therefore do not prove the old, nonzero receiver at `0x1403a69a2` has this table. Runtime table identity and generation remain `UNBOUND`.

The new object path calls `FUN_1404813f0` with two doubles and a context pointer. It writes context offsets `+0x80`, `+0x84`, and `+0x88`. No typed edge connects those values to preferred/downmix metadata, F13C, or a DDP-JOC/EC3 frame writer. Keep those joins `UNBOUND`.

## Evidence limits

Incoming package manifest SHA-256: `f21807af691ba950aa88c0964fe6091b5ed3daff44be4362c4aefe119fc0dcbe`; its PowerShell manifest verifier passed and the v11 parent hash matched. The Ghidra project records executable SHA-256 `4b4c624d94a5b47d97b62b28073e1bd351b2deeeea039a7ccfaa2af96562c8a7`, but the installed executable was not rehashed during that analysis. Treat the image identity as project-recorded, not freshly verified.

No encoder run, PCM, payload, key, or audio output was involved. The package preserves `writer=UNBOUND` and is not an audio-algorithm closure.
