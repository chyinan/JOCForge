# Potential AI Run147 DBMD negative-sentinel vector

This Wash addendum validates the Run147 input-side DBMD branch and joins it to the already washed Run147 DEE/OpenJOC metadata record. The authorized input identity, Run147 preflight, job XML, configured output basename, DEE completion log, and washed output metadata all refer to the same run.

Wash independently read only the 576-byte DBMD chunk at its hash-bound offset. The type-9 tuple is signed sentinel/hour -1, minutes 0, ticks 0, so the candidate helper's negative-sentinel branch applies; no candidate helper query was needed. No audio-data bytes or raw DBMD bytes were kept.

Run147's output metadata reports 6,448 E-AC-3/JOC access units, deployed compatibility PASS, ETSI strict FAIL, and render/playback NOT_RUN. This vector is input metadata plus output metadata identity; it does not infer coding behavior or exercise the positive DBMD delta branch.
