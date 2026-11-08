# ADM DDP silence duration: composite schedule boundary

## Scope

This evidence-only supplement covers XML field declarations, static seconds/frame-to-sample conversion, per-source count storage, and the composite sample schedule for prepend_silence_duration and append_silence_duration. It does not establish that zero-valued samples are materialized or that encoded timing, PCM, or channel/object layout changes as scheduled.

## Statically bound schedule

- The inspected XSD snapshot declares both fields optional as xs:string with lexical default 0. An earlier sealed package transcribed 0.0 from a different snapshot. Both explicit strings convert to zero; omitted-element overlay/materialization remains UNBOUND.
- The DDP property reader reads prepend and append by their named keys. The converter interprets decimal form as seconds multiplied by source sample rate and truncates toward zero; frame form requires a numeric timecode frame rate and rejects negative or fractional frame counts.
- Prepend and append counts are stored separately per source and copied into separate encode-context fields. The composite row stores prepend at +0x20, source start at +0x28, exclusive source end at +0x30, and append at +0x38.
- The schedule builder retains the selected source range [startSample,endSample), places its composite start at prior length plus prepend, and advances total length by prepend plus source range plus append. This binds schedule order, not the materialized audio samples.
- The encode loop requests a Reader frame through virtual slot +0x18, then passes the returned local frame object through an output-side virtual slot +0x08. The concrete Reader target, frame sample layout, zero-fill operation, output PTS/track length, and channel/bed/object consistency remain UNBOUND or NOT_OBSERVED.

## Handoff status

IMPLEMENTATION_READY=NO. Treat the schedule formula as a static scheduling fact only. Runtime remains NOT_RUN; the proposed prepend-only A/B has no Wash timeline GO, assigned attempt, or unique output/temp paths. Do not infer PCM placement from output byte size.

No DEE/JOCForge runtime, PCM, encoded payload, key, or license was accessed for this evidence slice. The handoff contains no product source or decompilation.
