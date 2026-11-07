# AtmosMezz data_rate selector constructor edge V2

## Static string table and frame-local write

The hash-bound Ghidra 12.2 trace finds an exact U32 lookup table: `384→0`, `448→1`, `576→2`, `640→3`, `768→4`, `1024→5`. In `FUN_1801535b0`, a workflow-parameter rate string is looked up in that map and the U32 result is written to frame-local `+0xD8`. The selected hash-bound job XML contains text `1024`; if that value reaches this lookup, the static result is selector code 5. No runtime value was captured.

## Conditional constructor transfer

The static branch `processor+0xFC8==2` carries the frame record through `FUN_1801746c0 → FUN_1801746d0`; `+0xD8` is passed to `FUN_180177e50/eb0`, whose constructor copies the U32 to provider state `param_1[0]`. `FUN_180178dc0` consumes that code and writes `open_config[10]`. The constructor-field transfer is bound only for the conditional `+0xFC8==2` branch.

The remaining same-run join is whether the selected job XML value enters the chosen runtime workflow-parameter map at `Priv+0x350` (`plVar9+0x6A` when `plVar9` is a 64-bit pointer), and whether that processor branch is selected. Both remain `UNBOUND/NOT_CAPTURED`. No DEE run, full encode, or writer behavior test was performed. Thus all six string mappings are statically known, but all-rate runtime/output support is not established.

## Provenance

Upstream manifest-list SHA-256 `9f94cf5e17bec7659f6defadb9145cdda6f28515f39ca3894031b49bbc6ef3d2`. Wash recomputed all three listed payload hashes, 16 referenced analysis artifact hashes, and the selected job XML/schema/source evidence. The Human Cat input SHA remains a previously recorded value, not rehashed in this pass. No raw logs/scripts, media, PCM, payload, key, or license are included.
