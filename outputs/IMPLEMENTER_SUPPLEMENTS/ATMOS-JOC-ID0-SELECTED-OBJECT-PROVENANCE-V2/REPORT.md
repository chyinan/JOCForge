# Atmos JOC selected ID-0 RenderBus provenance — corrected lineage

This revision binds each static claim to the source package that supports it.
It supersedes `ATMOS-JOC-ID0-SELECTED-OBJECT-PROVENANCE-V1` for provenance of
the local `FilterDDPJocEnc` constructor path.

## Bound edges

The renderer's RenderBus vector is selected by type ID `0`; the matching
object is forwarded to the factory as argument 5 and used as the receiver of
the `vtable+0x38` selector getter. The existing branch is `0` to
`SpatialCoderPlatform`, nonzero to `PreProcessorBypass`.

The local Atmos plugin-config path separately copies an opaque caller vector
from input `+0x18` to plugin storage `+0x138` with a `0x20`-byte element
stride. Its stage factory passes that field to `FilterDDPJocEnc` construction;
the nested `Priv` initializer does not consume the incoming vector argument.
This path does not construct the selected ID-0 RenderBus object or write its
`+0x58`. The opaque vector's type and identity with the renderer's RenderBus
vector remain `UNBOUND/NO`.

The base getter candidate returns `object+0x58`; the base constructor writes
that field from caller argument `param7`, with no literal default in that
constructor. The only direct constructor caller located within the examined
Atmos DLL is the `RerenderBusConfig` path, whose visible type IDs are `3/4/5`
and which is excluded from the ID-0 lookup. This bounds the in-image caller
search only; it does not prove the ID-0 object cannot be constructed by an
external module or another route.

## Implementation disposition

Keep the selected type-ID-0 getter result as an explicit typed runtime input
and preserve the conditional. Do not reinterpret the plugin's opaque vector as
RenderBusConfig, synthesize an ID-0 object, hardcode a selector default, or
infer that job XML supplies the value. The ID-0 concrete producer/writer and
per-run selector value remain `UNBOUND/NO`. The conditional MVP rule is not
blocked, but the exact historical branch and audio equivalence are unknown.

This supplement makes no claim about the separate per-block coefficient
stride selector or U7 packing contract.

## Scope and exclusions

Ghidra 12.2 static evidence only. The source packages reuse two authorized
ADM receipts and historical DEE observations; no new DEE run occurred. No
media, PCM, encoded payload, private key, license material, binary, raw
disassembly, or decompilation is included.
