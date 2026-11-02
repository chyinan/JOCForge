# Current Rust window provider supersession

The current selected Rust transform provider has file SHA-256 `963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c`. Its 256 window words hash to `097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2`, matching the native DEE table hash. At index 21 both contain F32 bits `3c0ea5aa`.

A current Rust one-hot fold test passes for input word 234 = `+1.0`, with other input and histories zero. The derived primary/companion outputs match the native static derivation. No DEE runtime or matched PCM comparison was run.

The earlier hash `45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23`, with index-21 value `3c0ea5a9`, belongs to a historical Rust provider and is superseded. The earlier coefficient-split package records that historical baseline; it is not a current parameter difference.

The matched ADM row 3 to core LFE lane, scratch history words 256..381, and channel-3 PCM causality remain unbound. This correction does not close the LFE algorithm comparison.
