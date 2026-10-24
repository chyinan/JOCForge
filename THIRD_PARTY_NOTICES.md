# Third-party notices

JOCForge uses `sha2` 0.10.9 from RustCrypto for SHA-256 over canonical transaction-state bytes. The selected license for this internal isolated implementation is Apache-2.0. The crate is sourced from crates.io and remains available under `MIT OR Apache-2.0`; repository: https://github.com/RustCrypto/hashes.

JOCForge uses `serde_json` 1.0.154 only for F01E provider JSON syntax parsing and private field access after the byte-identity gate. The selected license is Apache-2.0 from its `MIT OR Apache-2.0` offer; repository: https://github.com/serde-rs/json. Canonical serialization remains project-owned code.

JOCForge uses `num-bigint` 0.4.6 only for the F05R56 selected panner's exact-rational alternating trigonometry enclosure. It is configured with `default-features = false` and `features = ["std"]`; the selected license is Apache-2.0 from its `MIT OR Apache-2.0` offer; repository: https://github.com/rust-num/num-bigint. It does not read runtime files or own persistent codec state. Its locked transitive dependencies are `num-integer`, `num-traits`, and the `num-traits` build dependency `autocfg`.

JOCForge uses `roxmltree` 0.21.1 only to parse caller-supplied F01E.1 UTF-8 job XML into a read-only DOM. The selected license is Apache-2.0 from its `MIT OR Apache-2.0` offer; repository: https://github.com/RazrFalcon/roxmltree. The parser does not open files or perform network access; F01E rejects `DOCTYPE` and `ENTITY` declarations before DOM parsing. It is not used for general ADM graph parsing.

Locked transitive Rust packages are listed below. Checksums are the crates.io checksums recorded by Cargo.lock.

| Package | Version | License | Checksum |
|---|---:|---|---|
| roxmltree | 0.21.1 | MIT OR Apache-2.0 | `f1964b10c76125c36f8afe190065a4bf9a87bf324842c05701330bba9f1cacbb` |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | `f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53` |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | `3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71` |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | `4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600` |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | `59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280` |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | `78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a` |
| digest | 0.10.7 | MIT OR Apache-2.0 | `9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292` |
| generic-array | 0.14.7 | MIT | `85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a` |
| libc | 0.2.189 | MIT OR Apache-2.0 | `3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2` |
| itoa | 1.0.18 | MIT OR Apache-2.0 | `8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682` |
| memchr | 2.8.3 | Unlicense OR MIT | `cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98` |
| num-bigint | 0.4.6 | MIT OR Apache-2.0 | `a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9` |
| num-integer | 0.1.47 | MIT OR Apache-2.0 | `7ce2d95d4b3734dc35aa2f45e1aa22cd416814592a4f9d9205e11affd5b8e10b` |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | `071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841` |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | `985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9` |
| quote | 1.0.47 | MIT OR Apache-2.0 | `1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001` |
| serde | 1.0.229 | MIT OR Apache-2.0 | `4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba` |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | `67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48` |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | `e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348` |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | `e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6` |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | `a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283` |
| syn | 3.0.8 | MIT OR Apache-2.0 | `01016da373cd8f7ef12624f796309f5c31ba8d646dd08856c02cd741d823c622` |
| typenum | 1.20.1 | MIT OR Apache-2.0 | `b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20` |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | `d245f478577f809a851594d02313b640fb437e0bb33866753cff937863096954` |
| version_check | 0.9.5 | MIT OR Apache-2.0 | `0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a` |
| zmij | 1.0.23 | MIT | `29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b` |

JOCForge vendors and distributes no FFmpeg, ffprobe, OpenJOC or other external executable. JOCForge source is Apache-2.0; see LICENSE. External validation tools, if later provisioned, retain their own licenses and remain outside production encoding.

Previous notices and distribution provenance are preserved in the external recovery archive. This baseline makes no patent clearance claim and does not change third-party licenses.
