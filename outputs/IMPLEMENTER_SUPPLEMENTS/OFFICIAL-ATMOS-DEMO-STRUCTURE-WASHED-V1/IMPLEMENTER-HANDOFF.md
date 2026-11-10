---
status: reference-structure-bound-partial
topic: "User-designated 7.1 E-AC-3 JOC reference structure"
spec: "ETSI TS 102 366; ETSI TS 103 420"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, E-AC-3, JOC, D0, EMDF]
---

# 官方演示样本结构参考

## 样本身份

- User-designated basename: `Dolby-Atmos-DD+ 7.1.4.mka`
- Source SHA-256: `f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd`
- Wash source package manifest SHA-256: `13b691cc9dfc67667b1f02eb28f2257516d284992a58cc80a14569c5ef5ecd9d`
- Wash disposition: hash-bound full-track structure summary; source authenticity remains user-designated.

## Full-track structure measured

Across 23,033 Matroska audio packets, each 5,120 bytes, the sequential parser and independent frame-header parser agree on two 2,560-byte frames per packet, in this order:

1. I0: legacy independent AC-3 core, `bsid=6`, `acmod=7`, LFE present, six blocks, 1,536 samples, 2,560 bytes.
2. D0: dependent E-AC-3, `bsid=16`, `acmod=5`, LFE absent, `chanmap=0x1A00`, six blocks, 1,536 samples, 2,560 bytes.

The ordered pair is one AU per packet. The sample aggregate is 1,280 kbps. Its 48-kHz six-block frame extents are each 2,560 bytes (640 kbps per frame by frame extent).

The full track consistently reports a 7.1 layout: full-band locations `L,R,C,Ls,Rs,Lrs,Rrs` plus LFE. D0 has 2-byte JOC addbsi and complexity index 16. The D0 block-2 `skipfld` carries parsed EMDF payload IDs 11 and 14 in all 23,033 AUs; the observed skip-field extent is 301–377 bytes. OAMD content prefix reports 16 total objects: 15 dynamic objects and one LFE bed.

## Comparison to the 1,664-kbps target

The user's Flat7x target keeps I0 at 640 kbps and assigns D0 1,024 kbps, for two frames and a 1,664-kbps AU aggregate. The official sample has the same ordered I0+D0 shape and 640-kbps I0 frame extent; its observed D0 frame is 2,560 bytes / 640 kbps. Therefore, reuse the sample as a structural reference and treat the 1,024-kbps D0 extent as the separate target.

## Open target checks

- This package does not establish `joc_dmx_config_idx=1`; the addbsi field is 2 bytes and the JOC profile-index value was not independently parsed here.
- It does not inventory all object position/height history or validate ETSI Tables 55/56. The filename suffix `7.1.4` is not used as layout evidence.
- The observed sample's `skipfld` sizes fit within the 9-bit 511-byte syntax capacity. This is sample evidence for the 1,280-kbps stream; all-AU EMDF fit for the 1,664-kbps target remains `NOT_RUN`.
- No audio decoding, PCM generation, DEE run, private key/license access, or compressed packet/payload persistence was performed.

Keep this reference additive. Preserve the existing 5.1 production path and do not copy any reference payload or audio algorithm.
