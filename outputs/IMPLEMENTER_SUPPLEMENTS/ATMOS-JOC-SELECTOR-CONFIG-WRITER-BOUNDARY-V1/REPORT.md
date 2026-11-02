# Atmos selector configuration writer boundary

The selected RenderBus type-ID-0 object comes from the caller-provided config
vector and is forwarded to the selector getter at `vtable+0x38`. In this
Atmos DLL, the `RenderBusConfigBase` constructor writes selector storage
`object+0x58` from its `param7`; the examined base constructor does not set a
literal default. Its only direct caller found in this image is the
`RerenderBusConfig` path, whose type IDs are 3/4/5 and which is excluded from
the type-ID-0 Speaker Monitoring lookup.

This narrows the local contract: keep the selected ID-0 getter value explicit
and conditional. The DLL-local writer for the actual ID-0 object and its
per-run value remain UNBOUND, and the evidence does not establish a
user-facing prompt requirement. Do not use the Rerender default, hardcode a
branch, or infer XML/job ownership from this slice. The conditional MVP path
remains viable; exact historical audio branch remains unknown.

Static Ghidra evidence only. No binary or decompilation is included in this
Wash supplement.
