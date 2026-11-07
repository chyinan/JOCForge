# Preferred downmix enum serialization

## Bound DME host behavior

The hash-bound DME host schema and UI list for `DolbyDigitalPlusOnlineMedia.DownmixOptions.PreferredStereoDownmix` contain exactly three labels: `Stereo`, `Surround`, and `Not Indicated`. Their exact map is:

- `Stereo` → `loro`
- `Surround` → `ltrt`
- `Not Indicated` → `not_indicated`

The handler uses a case-sensitive exact Map lookup with no trim, case folding, numeric conversion, or fallback. Unknown labels throw and the enclosing config method returns error 400. The static schema and initial state both say `active=false, value=Stereo`. That is a static UI/template default, not a captured runtime or persisted user choice.

The mapping runs while `valuesToCombineMap` is built, before the active gate. A valid inactive value is normalized, then omitted. An active member is serialized inside `--encoder`: the Atmos branch emits `preferred_downmix_mode=<token>`; the other branch emits `ddp:preferred_downmix_mode=<token>`. Other active members remain colon-joined in insertion order. The outer `--encoder` property is still constructed when this member is inactive.

## Scope boundary

This package binds DME host source serialization only. It does not prove that a job XML schema accepts the same labels, that the user supplied one, or that DEE's native parser maps these tokens to a typed audio/BSI consumer. It does not support a fourth explicit enum such as `ltrt-pl2`. The native consumer, runtime value, and audio effect remain unbound or not run.

The vectors are source-derived input-only cases. No UI, DME, DEE, or A/B run occurred. No media, PCM, payload, key, or license content was read.

