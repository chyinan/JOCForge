# attempt-12 loader boundary

The bounded observer captured the error ID already passed by the host to its message formatter after failed library loads. All 44 values were Windows error 1114 (`ERROR_DLL_INIT_FAILED`): 40 observations from one wrapper group and 4 from the other.

The result classifies the failure as a DLL initialization failure. Four failed component basenames were observed, but this package omits their names and paths. Component files were present; the bounded direct/transitive PE import-name check found no missing import filenames. The observation does not identify whether a requested component or one of its dependencies failed initialization.

The target Atmos component did not load. No U/K/B/D events were produced. Attempt-11's direct `GetLastError` observations were empty; attempt-12 observed the existing error value at the host formatter argument. The prior static U-to-K ordering remains unchanged; runtime owner/generation was not captured.

Scope: Ghidra 12.2 read-only/no-analysis plus one bounded DEE diagnostic on an authorized 36-channel sample. The observer did not modify arguments or returns. No PCM, compressed payload, license, or key content was read. This package makes no global absence claim and does not claim a successful encoder load or output.
