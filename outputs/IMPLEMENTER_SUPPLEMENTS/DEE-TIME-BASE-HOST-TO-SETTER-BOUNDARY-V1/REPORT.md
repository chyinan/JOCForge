# DEE time_base host-to-setter boundary

## Bound static host path

The inspected StageRunner::Build path obtains an interface pointer from a descriptor vtable +0x30 call and stores it in StageRunner+0x270. RunAudioFilter later reads that member and supplies the interface to the generic SetProperty method. The observed keys on this route are dee_audio_filter_source and dee_audio_filter_target:<index>.

The factory target and the generic SetProperty target remain dynamic vtable calls. Neither target resolves in this evidence to the Atmos Priv+0x550 StringType setter. Thus this host path does not bind a selected job's time_base XML value to that field. The Atmos typed field, static default producer, and sample-range consumer are separately established by parent Wash receipts and are not re-traced here.

## First break and limits

The earliest unresolved edge is the descriptor-vtable +0x30 factory returning the concrete StageRunner property adapter; the subsequent vtable +0x8 SetProperty target is also unresolved. The dynamic key/value and runtime owner/generation remain unobserved. No DEE/XML job ran, and no process or media was accessed.

The upstream checksum-list file SHA-256 is ab65d0d6774e842e59d3dc9f85e33eb1aa66474635d91b270dc8cd0dd144c8dc. Its checker passed normally and under python -O. This supplement excludes raw Ghidra logs and source scripts.
