# F06 BSI U16 active measurement-update observation

A single bounded scalar/pointer capture on the authorized input observed the measurement-update path. The task-local ASCII alias SHA matched the original authorized input SHA. At context entry, mode was 0 and update_flag was 0: the mode-one initializer was not taken, while the update-flag-zero measurement update was selected. The serializer read signed U16 zero at BSI owner +0x34 using the same normalized owner token on that thread.

This is a native value observation for one measurement callback only; it does not establish the constructor-time value or a global default. Static evidence independently shows both confirmed fixed-block setters place low U16 zero at +0x34 and the source qword vector is not read by them.

No final EC3 file was present at the last check, and no DEE exit-code receipt was captured. No PCM or EC3 bytes were read. This is not a complete encode or output validation.
