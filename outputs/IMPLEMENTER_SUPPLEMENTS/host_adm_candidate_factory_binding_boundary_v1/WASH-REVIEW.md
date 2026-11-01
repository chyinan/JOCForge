# Wash review: AtmosMezz factory candidate is not R138 ADM

Ghidra statically closes a candidate internal factory method to an AtmosMezz input-object constructor, and that object's supported input-type label is audio.atmos_mezz. R138's hash-bound configuration input type is audio.adm. The candidate is not bound to the host audio_input getter slot or to the R138 typed input owner/generation.

The generic host audio_input getter-table registration mechanism is separately closed, but no evidence joins this candidate to an actual returned table entry for R138. The R141 observation did not capture a registry slot, typed input owner, reader vtable, or generation join. Do not change the user-facing R138 input type to AtmosMezz based on this internal class candidate.

This is static Ghidra evidence only; no native job or media payload was used.
