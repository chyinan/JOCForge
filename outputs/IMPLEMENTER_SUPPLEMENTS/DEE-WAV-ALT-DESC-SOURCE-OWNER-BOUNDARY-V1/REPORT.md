# WAV alternate descriptor source-owner boundary

## Bound static transfer

The upstream Ghidra receipt follows a Factory vtable +0x20 method whose param_4 shared-owner/control-block pair is copied and passed into an alternate descriptor-source method. That method invokes the source object's vtable slot +0x60, uses the returned value to initialize a separate newly constructed AdmFileStorageDesc at +0x38, and later forwards that field value to a descriptor constructor path.

This binds a source-derived value flow but not the source pointee's concrete type, owner, or generation. The newly allocated descriptor is not proven to be the same instance used by the selector-3 lower-storage path.

## First break

The concrete caller/producer of the Factory virtual +0x20 method is unresolved. Therefore the source pointee identity and generation remain UNBOUND. No conclusion is made about the selector-3 descriptor's +0x68 nonzero limit writer. The user-supplied V2 parent manifest was unavailable and remains a hash reference only.

The upstream package seal and normal/python -O checker passed. No DEE run, media/PCM/payload/key/license access, or output occurred. Raw Ghidra material is excluded.
