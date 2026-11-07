# Composite silence formatter boundary

Reference-only Wash supplement. The observed path formats CompositeMaster rows for a diagnostic message; it does not establish silence insertion or audio effect. `IMPLEMENTATION_READY=NO`.

`FUN_1801849c0` receives a 0x40-stride row vector and output string. It computes the count as `(end-begin)>>6`, formats an index prefix, calls `FUN_180184960(row_base,&local_40)`, and appends that returned string. The body does not directly read row `+0x20` or `+0x38`. Its caller formats the returned string into `Initializing Composite Master: ...` for logging.

The first unresolved field edge is `FUN_180184960`: its body was not inspected, so reads of silence qwords and their types remain `UNBOUND`. No Reader, PTS, PCM, sample insertion, or audio effect follows from this formatter evidence.