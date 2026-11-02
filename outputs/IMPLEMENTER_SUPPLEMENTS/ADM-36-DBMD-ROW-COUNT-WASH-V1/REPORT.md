# ADM 36-row DBMD metadata receipt

## Result

For the user-authorized `D:\First Love.wav`, the current metadata-only read found:

- RIFF/WAVE `fmt `: PCM, 36 channels, 48 kHz, 24-bit.
- CHNA: 36 tracks and 36 UIDs.
- DBMD: 576-byte payload, sections 7, 9, and 10; section 10 declares 36 track records.
- The DBMD record count equals both the WAVE physical channel count and CHNA track/UID counts for this sample.

This is a one-sample metadata observation, not a general rule for arbitrary ADM. It supports constructing a correctly dimensioned 36-row metadata fixture. It does not prove that the DBMD records are joined to the same production `PreparedJobInput`, nor does it prove a 36-row native audio run or output.

## Read boundary

The RIFF parser sought over the 1,334,998,044-byte `data` chunk. It read only the container header and `fmt `, `axml`, `chna`, and `dbmd` metadata chunks. `pcm_bytes_read=0`; no audio was decoded or exported. The complete source-file SHA-256 was not recomputed in this pass. Identity is bound here to the authorized path, file length, and hashes of the inspected metadata chunks.

## Reproduction detail

The DBMD parser starts after the four-byte DBMD preamble, validates each section's declared length and checksum, and reads the little-endian U16 at section 10 body offset `+4` as the number of track records. For this sample the value is 36. No per-track values are included in this package.

## Scope

Use this receipt only to correct the 36-row synthetic metadata fixture dimension. Keep field-level DBMD-to-CHNA mapping, same-resource/generation joins, normal callback execution, DSP, and E-AC-3/JOC output status separate.
