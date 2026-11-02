# Current Rust window provider direct verification

Wash directly rehashed the current public transform-parameter JSON file and recomputed its 256 F32 window-word digest. The provider file hash is `963a176760f7c7d43dba138553717b7d5273a9457a574f658ee73eeb62e7da5c`; the packed window words hash is `097ea95140024fd143ed18a78299fa72d69bce1c8e0d356aed2efff2a6a099a2`. Both match the native DEE window-table hash. Index 21 is `3c0ea5aa` in both.

The earlier 12 one-ULP split compared the native table with a historical Rust provider whose hash was `45e8cbe4c07bd1b59bc009ed6ac77d7911a3a309b01308131da97e640da95b23`; that provider is superseded. It is not a current Rust-vs-DEE parameter difference.

This direct parameter check does not establish that the matched ADM row 3 reaches the native core LFE lane or identify the source of scratch history words 256..381. The LFE transform and PCM root-cause comparison remain open. No Rust source code, PCM, encoded payload, private key, or license material is included.
