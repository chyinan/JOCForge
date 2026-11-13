# ProEncoder source-range method-set boundary

## Wash-approved observation

On the hash-bound candidate image (`2b7da4835d60fc538caf49cd0ef27a4a10d2486bc03b754b41b5bea4bb51fead`), the selected same-object `DDPJocProEncoder` method set covers vtable slot 1 wrapper/worker, slot 2 wrapper and two selected helpers, plus the slot 3 leaf getter.

Within that selected set, the worker reads the range locations at `this+0x190/+0x198` through its `this+0x140` subobject (`+0x50/+0x58`), but no direct write to those range locations was observed. Apparent writes to the same numeric offsets were stack-frame slots and were kept separate. The slot-3 method reads a wrapper-state flag at `+0x101`; it does not populate the worker range fields.

## Implementation boundary

This result covers only the listed same-object methods. It identifies a likely boundary where source ranges must be supplied or populated elsewhere; the external range populator, ADM row/role mapping, generation join and D0 audio algorithm remain unbound. A caller-owned D0 span input with typed source identity is an implementation candidate seam, not proof that valid spans or an encoder are available. Keep the candidate disabled until source lineage and audio generation are supplied. No conclusion applies to methods outside the scanned set.

This package contains a sanitized summary and source receipts only. It contains no native code, decompilation, media, PCM, or compressed payload.
