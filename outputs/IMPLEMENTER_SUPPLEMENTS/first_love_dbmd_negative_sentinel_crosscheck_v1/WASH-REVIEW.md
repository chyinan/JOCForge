# First Love DBMD negative-sentinel cross-check

This Wash addendum independently reads only the 576-byte DBMD chunk from the hash-identified, user-authorized First Love input. The payload SHA-256 matches the sealed source package, and the type-9 tuple fields at the reported record-body offsets match signed hour/sentinel -1, unsigned minutes 0, and signed little-endian ticks 0. No audio-data bytes were read and no raw DBMD bytes were persisted.

The candidate helper accepted the typed 7/9/10 records, passed their checksum checks, did not take decode fallback, and reported StartTime=FFOA=3610.0 seconds. The candidate rule predicts the same result: the negative sentinel suppresses absolute source FFOA writing, preserves relative FFOA -1.0, then returns StartTime + max(relative, 0).

This is the second authorized sample where the candidate helper's negative-sentinel branch explains equal StartTime/FFOA. It does not exercise the nonnegative DBMD time branch, establish a universal rule, or run DEE/codec/audio output. No second sample was selected, per the bounded task.
