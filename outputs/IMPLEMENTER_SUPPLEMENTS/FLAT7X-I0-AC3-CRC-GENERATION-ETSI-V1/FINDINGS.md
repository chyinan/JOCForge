# Flat7x I0 AC-3 CRC1/CRC2 generation contract

## Approved structural rule

ETSI TS 102 366 V1.4.1 §4.4.1.2, §4.4.5.1–4.4.5.2 and §6.10.1 specify the two AC-3 CRC locations, coverage, polynomial and zero-residue checks. This Wash addendum derives the leading CRC1 field value and cross-checks it with an independent 16x16 GF(2) solve. The rule is a non-audio integrity operation and applies to caller-provided frame bytes.

- CRC parameters: polynomial x^16+x^15+x^2+1 (0x8005), zero initial register, MSB-first, no reflection, no final xor; syncword is excluded from CRC input.
- CRC1: U16 at bytes [2,4). For a 2560-byte target frame, the ETSI 5/8 boundary is 800 16-bit words = 1600 bytes. The checked span is frame[2:1600], including the CRC1 field and excluding the syncword. With S=frame[4:1600], derive C1 = CRC(S) × x^-(16+8|S|) mod P. The candidate is accepted only when CRC(frame[2:1600]) == 0. It is not the naive CRC(S).
- CRC2: U16 at bytes [frame_len-2, frame_len). The candidate field is CRC(frame[2:-2]); the full check is CRC(frame[2:]) == 0.
- crcrsv: one bit immediately before CRC2. If the computed CRC2 equals syncword 0x0B77, an encoder may toggle crcrsv and recompute CRC2 to avoid the collision.

## Synthetic-only validation

Four deterministic byte patterns verify target geometry, word-aligned and odd-word 5/8 truncation boundaries, and the crcrsv collision adjustment. All are synthetic CRC geometry; none is a valid or decoded AC-3 bitstream. Reference outputs contain only scalar CRC values, residuals, extents and frame hashes. No frame bytes or audio data are included.

This addendum authorizes only the CRC field generation/validation rule. It supplies no AC-3 audio blocks, no PCM, no full I0 source and no end-to-end output permission; production_allowed=false.
