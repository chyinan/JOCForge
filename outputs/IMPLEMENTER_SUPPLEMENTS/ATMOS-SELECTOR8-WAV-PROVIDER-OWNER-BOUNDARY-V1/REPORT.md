# Conditional WAV provider and selector-8 owner boundary

## Wash disposition

The upstream sealed static analysis package was independently hash-checked. Its Ghidra 12.2 evidence supports a conditional provider path and the selector-map writer behavior below. It does not identify the provider selected by the current job, the object returned as selector-8 owner, or a production value for its `+0x18` field.

## Bounded static facts

- In the analyzed Atmos DDP plugin image, extension `wav` maps to type ID 3; the type-3 BWF master-open callback constructs an `atmos_storage_master_s` provider.
- If that provider is selected, slot `+0xa0` returns a record from its selector-keyed map; slot `+0xa8` writes the record.
- In the writer's normal branch (`param_3[0] == 0`), its three qwords are copied into the map value. Therefore the returned record's qword 0 comes from writer argument `param_3` qword 0 under that branch.
- A separate sentinel branch (`param_3[0] != 0`) does not copy qword 0; it marks the map value's first byte.

## Required boundaries

The analyzed files do not show the selected job's extension or selected provider instance. The selector-8 writer callsite for that instance, the type of the qword-0 pointee, and that pointee's `+0x18` constructor/first writer remain `UNBOUND`. The provider's own `+0x18` initialization is a different object and cannot supply that answer. The selector-8 owner is not joined to profile selection here.

Treat the listed branch behavior as a static conditional only. Do not use it to assign a selected-job owner or a default. The vector is `NOT_RUN`; no DEE runtime, media, PCM, key, or license was accessed.
