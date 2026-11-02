# Matched-run DEE LFE pointer and first-sample edge

This addendum narrows the matched-run channel-index-3 investigation to one typed DEE sample path. The authorized ADM metadata labels physical row 3 / track 4 / UID ATU_00000004 as RoomCentricLFE, and the user confirmed that the extracted DEE output channel index 3 is the low-frequency track.

In the hash-bound DEE Atmos path, a six-channel request's source pointer table entry 3 is copied into the core API pointer slot 3. At the first sample-valued mixer, internal LFE group 3 takes unity accumulation; the weighted coefficient branch applies only to groups 4–7. Later staging also copies sample words unchanged.

The remaining unbound link is the same-run owner/generation join from ADM UID ATU_00000004 to the raw request pointer table. The DEE LFE transform after the core API boundary and its numeric output are also not identified. Current Rust selects external row 3 and routes it through a dedicated seven-bin F08 LFE transform.

The smallest next experiment is one low-frequency, single-row stimulus at the first shared sample-processing seam; compare the native output scalar with Rust F08 while holding all other 5.1 inputs at zero. This package does not authorize a guessed gain or remap.
