# Wash review: open-time reader query vs PCM sample read

Ghidra distinguishes two `+0xC8` candidates. A master-open caller reaches one `+0xC8` indirect call that uses a returned reader object's `+0x48/+0x50` values to aggregate extent and set the master channel count. That open-time query has no PCM row-table or actual-frame-count argument. Separately, the candidate sample-read interface accepts a borrowed row table and reports requested/returned frame counts, but its runtime target was not bound to the sample-6 full-file activation.

The bounded caller search found only one `+0xC8` owner reachable from the master-open chain; other candidates were not joined within the bounded search. Do not infer that the open-time extent query is the sample-read method. The R3 full-file receipt lacks helper PID, master token, reader vtable/target, row table, and frame request/return values.

The minimum next observation is one same-job helper sample-read call capturing those scalar/owner tokens only, with no row data or PCM values.
