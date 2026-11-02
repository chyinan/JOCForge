# E-AC-3 AU0 exponent parser failure: first syncword confirmed

## Confirmed parser failure

The exact old Rust artifact `sample6-keyless-full-20261101-10.ec3` is 17,395,712 bytes, SHA-256 `593b6f866bf4396d25cd7b6307515f965f69e9884ce46599675c6f5d0a8c0473`. A bounded in-memory read of its first 4096 bytes found big-endian syncword `0x0b77` at byte 0. FFprobe/FFmpeg 8.1 identify raw E-AC-3 at 48 kHz, 5.1(side); the first AU is 4096 bytes/32 ms, and AU0 fails decoding with `exponent -1 is out-of-range`. FFmpeg exits before writing PCM. The ADM input and Rust build/run identity for this old artifact remain unbound.

## Best-supported next implementation check

The raw demuxer has found a valid E-AC-3 packet and syncword. The current evidence points to AU0's audio-block exponent producer or the cursor/strategy selection before the first U4, not initial syncword detection. The washed DEE writer contract is high confidence for a nonzero-strategy lane writing one U4 then `N` U7 values on one cursor (`4 + 7*N` bits). It does not contain the pre-U4 header/block cursor for this Rust artifact.

On AU0, capture the first nonzero-strategy fullband lane's cursor before U4, block/lane, actual U7 count `N`, and raw U4/U7 codes. Use the 4+7N contract to identify which code is being decoded as `-1`; compare the next bit-word boundary with DEE's frame-size contract. After a candidate fix, parse a fresh same-input output to a null sink and confirm AU0 passes without saving PCM.

Confidence is high for this exact artifact's parser failure and first syncword; medium-low for the specific Rust root cause. This package reports no intra-packet bit offset and does not bind the artifact to an ADM or source commit.

## Exclusions

No EC-3 bytes, PCM, DEE binary, disassembly, decompilation, private key, or license material is included. No DEE run or Python prototype search occurred.
