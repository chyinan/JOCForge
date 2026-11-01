# Run145 / Run147 processing-module identity boundary

Wash independently rehashed the upstream package and retained Run145/Run147 preflight, job, DEE-log, and OpenJOC metadata. For both runs, configured output basename matches the OpenJOC inspection input basename, and the DEE log records encoder completion.

Neither run has a per-run loaded-module receipt. The logs and OpenJOC metadata do not bind the processing module path or loaded image SHA-256 to the exact DEE process. Neither run is therefore joined to the separately hash-identified target module; both remain UNBOUND / NOT_EVALUATED.

This is an operational evidence boundary only; it does not change or specify E-AC-3/JOC behavior. No new DEE job, debugger attach, EC-3 payload read, or PCM read was performed.
