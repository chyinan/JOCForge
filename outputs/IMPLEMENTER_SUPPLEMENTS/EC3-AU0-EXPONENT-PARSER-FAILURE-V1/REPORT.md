# E-AC-3 AU0 exponent parser failure

## Confirmed parser failure

The single user-designated generated EC-3 artifact `sample6-keyless-full-20261101-10.ec3` was 17,395,712 bytes with SHA-256 `593b6f866bf4396d25cd7b6307515f965f69e9884ce46599675c6f5d0a8c0473`. FFprobe 8.1 recognized raw E-AC-3 at 48 kHz, 5.1(side). Its first packet starts at byte 0, is 4096 bytes, and lasts 32 ms. FFprobe and FFmpeg 8.1 both failed while decoding AU0 with `exponent -1 is out-of-range`; FFmpeg exited 1 and no PCM reached the null output. The exact intra-packet bit offset was not reported.

This confirms that the old artifact is not decodable by that parser path. Adobe Audition failure remains user-reported and was not independently tested. The artifact's ADM input and Rust build/run identity were not bound in this package.

## Best-supported fix direction

Prioritize the first AU's E-AC-3 exponent producer or bit cursor; verify that U4/U7 counts and bit cursor match the selected lane's syntax before emitting the audio block. The new U4/U7 DEE supplement gives a concrete static contract for this comparison. Preserve frame size and buffer-word consistency; the parser already found the first 4096-byte E-AC-3 packet, so the first observed failure is in the audio block, not initial syncword detection.

The DEE Ghidra evidence is partial: a six-block frameset boundary, `bse_frmsiz`/buffer-word count check, a header-record path containing `0x0b77`, and a descriptor-driven bit-span writer were identified. The exact on-wire field order and runtime bit count were not recovered. This points to a bounded comparison, not a proven Rust root cause.

## Exclusions

No EC-3 bytes, PCM, DEE binary, disassembly, decompilation, private key, or license material is included. The artifact is represented only by its size and SHA-256. No DEE run or Python prototype search occurred.
