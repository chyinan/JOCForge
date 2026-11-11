# Flat7x I0 AC-3 BSI continuation contract

## 结论

ETSI TS 102 366 V1.4.1 §4.3.0–4.3.5、§4.4.2.11–4.4.2.31 与 §4.4.5.1–4.4.5.2 明确了 I0 AC-3 的 syncframe 顺序、header 之后的 BSI tail、六个 audio blocks、auxdata 和 errorcheck 的排列。目标 acmod=7 不进入 dual-mono（acmod==0）字段分支。

已批准的 I0 header helper 在 compr 之后返回 caller cursor（无 compr 为 bit 62，有 compr 为 bit 70）。紧接的 BSI tail 由 caller-owned bit span 表示，范围从该 cursor 到 BSI 末端；其中可选字段按标准的 presence flag 与长度条件编码。BSI 之后依次是 audblk[0]…audblk[5]、auxdata、errorcheck。

## 2560-byte I0 frame part order

1. Syncinfo：syncword U16、crc1 U16、fscod U2、frmsizecod U6。
2. BSI：已批准 header segment 到 compr；随后 langcode U1/可选 langcod U8、audprodie U1/可选 mixlevel U5+roomtyp U2；acmod=7 时没有 dual-mono Ch2 子字段；然后 copyrightb U1、origbs U1、timecod1e U1/可选 timecod1 U14、timecod2e U1/可选 timecod2 U14、addbsie U1/可选 addbsil U6+addbsi（addbsil+1）字节。
3. 六个 caller-owned audio-block spans，严格按 index 0..5。
4. Auxdata：auxbits span、按 auxdatae 条件提供 auxdatal U14、auxdatae U1。
5. Errorcheck：crcrsv U1、crc2 U16；CRC2 为 frame 最后 16 bits，frame extent 20,480 bits。

CRC field positions and coverage are standard-bound, while this package leaves generated values caller-owned: crc1 is at syncinfo bit 16 and covers the first 5/8 of the syncframe excluding syncword; crc2 is the final U16 and covers the complete syncframe excluding syncword. ETSI specifies polynomial x^16+x^15+x^2+1, zero initial register, and a zero residue at each applicable check boundary. CRC1 generation is not the naïve CRC of a prefix; its value is chosen so the register is zero at the 5/8 boundary. The crcrsv bit may be inverted to avoid crc2 equaling the syncword.

## Implementer boundary

The structural assembler may accept an opaque, length-bearing BSI-tail span ending at audblk0, six indexed caller-owned block spans, and an optional caller-owned auxdata span, then reserve the final errorcheck fields. A supplied CRC U16 is a caller value only; it becomes verified only after the appropriate residual check is executed and recorded. This contract does not provide I0 audio-block bytes, PCM, an AC-3 encoder, XML value mapping, or a same-generation source receipt. production_allowed=false.
