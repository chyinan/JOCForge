# Atmos DDP Line DRC: static profile mapping

Scope: hash-bound static analysis of the selected Atmos DDP plugin. The XML field is an exact string enum. The verified mapping below is conditional on the configured value reaching the selected plugin and the inspected mode-1 path. It does not establish that path or serializer guards for a particular run.

| Token | Profile-tree U32 | DDP_COMPR U32 | Three bits if serialized |
|---|---:|---:|---|
| film_standard | 1 | 1 | 001 |
| film_light | 0 | 2 | 010 |
| music_standard | 3 | 3 | 011 |
| music_light | 2 | 4 | 100 |
| speech | 4 | 5 | 101 |
| none | 5 | 0 | 000 |

The mappings use exact key/value entries, not enumeration order. Values are categorical integers; no time units or rounding apply. On the inspected conditional route, numeric zero for none remains an explicit parameter; it is not automatically an omitted parameter.

The plugin descriptor default is film_light; a separate host queue descriptor reports music_light. Effective omission behavior is UNBOUND. XML-to-selected-plugin runtime transfer, selected mode, serializer activation, decoded PCM and audible effect remain unobserved. The A/B inputs are input-only; no output or runtime PASS is asserted. No media, native code, decompilation or key is included.
