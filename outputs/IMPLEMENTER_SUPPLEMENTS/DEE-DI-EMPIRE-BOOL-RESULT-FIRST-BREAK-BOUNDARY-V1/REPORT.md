# EmpireLoudness Bool result boundary

The already bound setup Bool reaches an engine subcontext field conditionally: the selected mode branch writes the Bool-derived value to subcontext +0x164; other mode values choose constants. Runtime mode was not captured.

The selected downstream consumer receives the same engine subcontext but reads only +0x08, +0x10, +0x18, and +0x20. Its next call forwards only +0x18 and +0x20. No typed transfer of +0x164 appears on this selected edge. The existing Float32 result write has a separate bound owner path, but no Bool-to-result causality is established.

First break: the selected consumer-to-next-call argument edge. Result causality remains UNBOUND. This is one bounded static path, not a global absence claim. No DEE run, runtime capture, media, PCM, payload, key, or license access occurred.
