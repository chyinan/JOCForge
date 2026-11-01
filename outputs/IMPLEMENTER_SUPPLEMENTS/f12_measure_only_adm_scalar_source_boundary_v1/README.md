# F12 measure-only source and user-input boundary

For the hash-bound My jealousy ADM sample, Wash independently confirmed the declared AXML chunk hash and no loudness, DialNorm, or metering fields. The sample job selects measure_only with metering_mode 1770-4, dialogue intelligence off, threshold 100, and no custom_dialnorm element. The reported schema states custom_dialnorm zero does not override measured loudness. Thus the evidence supports measurement under the job method settings rather than a supplied AXML loudness scalar for this sample.

Ghidra traces the measurement dispatcher to a per-source float aggregator initialized at -120.0. It walks 32-byte result entries, reads a float at +0x14, and keeps the strict maximum. One native run logged -17.093924. The log does not state units; the meter-vtable conversion, AXML programme-to-source entry join, per-source DSP, and any meter-side scaling or rounding remain open.

This is not a general measurement unit or BSI DialNorm mapping. It does not justify asking the user for a scalar, and it does not specify the final encoded field value.
