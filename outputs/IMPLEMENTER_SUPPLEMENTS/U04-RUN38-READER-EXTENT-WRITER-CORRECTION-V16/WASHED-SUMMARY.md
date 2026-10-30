# Run38 main reader extent writer correction

Disposition: MAIN_READER_EXTENT_FIELD_WRITE_BOUND; ACCESSOR FORMULA AND CLOCK JOIN OPEN. This package withdraws the cross-path join made in v15 and records a same-object run38 writer.

## Provenance and verification

Source v16 manifest SHA-256: 0194bc328e2a6c0a3db1966fc530813236359993fead62a6f1b96895427fca03.
NATIVE-READER-EXTENT-WRITER-CORRECTION.json SHA-256: 41d3912376a77736a821e82d2452e76b812aef5be8d1999fb476e1b2d5bc34f4.
Run38 job XML SHA-256: 4fc2df4dfc4ccf96555be749e1ecc7aebd26887940d008643e88f4628294f18d.
Authorized input BWF SHA-256: 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1.
Reference EC3 SHA-256: 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686 (262144 bytes).
Source checker passed normal and optimized Python; Wash rechecked payload hashes. No audio samples, raw pointers, code, addresses, license, or key are included.

## Same-object run38 observation

The main source construction recorded one 64-byte row: prepend=0, start=4320000, end=4418000, append=0. A writer observation on the main reader object's extent field saw the constructor initialize it to zero, call the master-duration accessor, then store the returned 98000 into that same field. The same reader object entered the outer loop with extent 98000 and cursor 0.

The first frame requested 1536 samples; the span helper returned 1536; the reader cursor advanced to 1536; the same descriptor pointer reached the encoder with 38 channel-configuration entries, sample span 1536, and event count 38.

## Corrected clock boundary

The v15 observation of master-duration return 6522906 came from an earlier prepass reader branch. It is withdrawn as a join to this main encoding reader. Run38 binds the accessor return and state-field write at 98000 for the main reader.

The accessor's internal formula/unit semantics remain UNBOUND. end-start is numerically 98000, but this equality is not a typed clock mapping or proof of the full formula. Reader-relative cursor to ADM sample position/host frame, CHNA/global source ID, frame pointer table writer/capacity, and U04/fit/spatial generation joins remain UNBOUND. No model comparison, Gate1, or product acceptance is claimed.
