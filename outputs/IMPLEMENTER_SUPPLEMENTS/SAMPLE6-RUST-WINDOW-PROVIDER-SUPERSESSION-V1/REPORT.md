# Current Rust window provider supersession

The current selected Rust transform provider is bound by provider-file SHA-256 `963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c`. Its 256 window words hash to `097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2`, matching the native table hash in the washed static split package. At index 21, both current Rust and native use F32 bits `0x3c0ea5aa`.

The current Rust one-hot fold regression passes: 256 input words, only input index 234 set to F32 `+1.0`, and 128-word primary and companion histories set to positive zero. Output primary index 107 is `0x3c0ea5aa`; companion index 106 is `0xbc0ea5aa`.

The earlier Rust words hash `45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23` and index-21 value `0x3c0ea5a9` belong to the old provider from source commit `216326f622e66f1f7b255dbe7a8bcfb8ca473ce2`. That historical split was corrected in the current provider. The prior split package does not describe a current Rust parameter difference.

The matched ADM row 3 to native core LFE lane, scratch history words 256..381, and PCM residual causality remain unbound. This receipt does not establish a matched LFE output or explain the measured channel-3 residual. It contains no PCM, encoded payload, native source, disassembly, private key, or license material.
