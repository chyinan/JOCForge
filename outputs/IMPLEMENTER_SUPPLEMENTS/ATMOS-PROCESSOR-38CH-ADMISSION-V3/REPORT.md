# AtmosProcessor 38-channel admission with receipt audit

The selected Atmos path builds a CompositeMaster U16 channel count at
`CompositeMaster+0x1A`. The reader exposes the same field as
`master_descriptor+0x0A`; it writes that value to descriptor `+0x08` and
passes the same master pointer at `+0x28`. Processor initialization reads the
master U16 into state slot 4, and the nonzero-frame guard compares descriptor
count with that state value. No fixed 36-channel cap appears in this guard.

For the authorized 人是猫 metadata (10 DirectSpeakers bed + 28 object tracks),
the static writer can produce 38 either by copying a child count of 38 or by
the special `10 + max(object-marked records)` rule when 28 object records
carry the marker. The path does not directly read BWF `fmt`; the `fmt` to child
count/object-marker runtime relation remains open.

Existing same-sample receipts were audited: none captures the target U16.
R141 carries an external license argument, and the existing pointer probes
stop before the count writer. No new runtime attempt was made because no
license-free pre-encode human-cat session or harness is documented. Treat 38
as statically admissible, not as a verified runtime value or full encode.

The selected profile uses 1536-frame capacity. Initialization accepts 512 or
1536, while zero is a flush only after initialization. No BWF, PCM, payload,
or private license contents were read.
