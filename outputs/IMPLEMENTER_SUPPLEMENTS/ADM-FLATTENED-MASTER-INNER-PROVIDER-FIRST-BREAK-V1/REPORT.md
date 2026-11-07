# FlattenedMaster inner-provider handoff boundary

## Bound static route

In the inspected resolver's nonzero param8 branch, a factory creates an atmos_storage_flatten_master wrapper. The wrapper's virtual slot +0xA0 forwards only its nested-provider field at wrapper+0x60 to that provider's own slot +0xA0. Setup stores the format descriptor separately at wrapper+0x58 and the nested provider at wrapper+0x60. No same-owner relationship is established between those pointers.

The provider call returns a three-qword record in the reused parent path; qword 0 is consumed there as an owner pointer. The concrete inner provider target and the concrete type of qword 0 remain UNBOUND. No typed join connects that qword-0 owner to the separate format descriptor or to a selected ADM reader/profile.

## Limits

The nonzero param8 route is conditional; runtime param8 selection was not observed. The param8==0 source-copy route remains separate. The internal U16 channel count at format+0x0A is inherited from a parent receipt and was not re-derived here. The current installed DLL was not rehashed; the target identity is project-recorded. No DEE run, audio/media read, implementation access, or code change is claimed.

The upstream manifest SHA-256 is e859bccf68ea0aaf1a0353652b4e3905f6fe7088f8e111043ce47d0a7965a5cf and its package seal file SHA-256 is 8b7a660f7cf1ca323b4f2525207e1d3fc6b7bce3d5c5273eda5d8262215b9f73. Its checker passed normally and under python -O. This supplement excludes code listings, source, media, PCM, payload, key, and license material.
