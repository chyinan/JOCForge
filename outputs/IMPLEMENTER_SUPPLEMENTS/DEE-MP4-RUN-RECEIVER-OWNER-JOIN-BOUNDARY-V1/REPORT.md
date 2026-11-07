# MP4 Run receiver owner boundary

Reference-only Wash addendum. It confirms receiver flow inside a Run activation, not the constructed object identity. `IMPLEMENTATION_READY=NO` for MP4 output selection.

`Mp4::Run` (`FUN_140148e90`) directly calls `Mp4::InitPlugin` (`FUN_140147140`) with the same `param_1`; InitPlugin reads its property data at `+0x1d8`. The setter candidate `FUN_1401496e0` stores `output_format` at receiver `+0x3b`, but it has no code-call references. The Run function also has no code-call references; an untyped slot at `0x1402f2788` stores its function address and has no references.

Therefore the setter's receiver/member is not joined to the Run/InitPlugin receiver and no constructor/factory owner is identified. The first break is Run entry → caller/factory owner → constructed MP4 receiver. Muxer/output behavior remains `UNBOUND`/`NOT_RUN`.