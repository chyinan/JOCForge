# RF mode DRC StringType to encoder boundary

## Disposition

RF_STRING_ENUM_DESCRIPTOR_AND_SCHEMA_DEFAULT_BOUND; PARSED_VALUE_TO_ENCODER_OR_BSI_SELECTOR_UNBOUND; RUNTIME_NOT_RUN.

In dee_audio_filter_ddp_atmos.dll, FUN_180136220 builds the property token drc:rf_mode_drc_profile. The property descriptor at param_1 +0x730 is initialized as dee::Types::StringType and carries six declared strings: film_standard, film_light, music_standard, music_light, speech, none. Its schema default is film_light; the selected runtime value was not observed.

The inspected plugin encode path FUN_1801475f0 -> FUN_180145380 -> FUN_180147d10 -> FUN_1801535b0 has no direct reference to descriptor +0x730 in this scoped slice. In the separate dee_ddp_encoder.exe image, a DRC U8 selector at state +0xac8 feeds profile selection and DRC row consumers. No typed owner/call edge joins the RF StringType property to that selector. No enum ordinal or audible effect is inferred.

## Independent Wash checks

Wash recomputed the source manifest, all four payloads, manifest checksum/seal binding, all eight source-export hashes, both parent manifest/report pairs, and both target binary hashes. The source agent reports normal and python -O checker PASS; Wash did not execute the checker.

No runtime/DEE run, PCM/payload/key/license read, or implementation contact occurred. Raw Ghidra scripts and traces are excluded from the supplement.

## Implementation boundary

The descriptor type, declared options, and schema default are established. The actual parsed XML value and its transfer to an encoder parameter or BSI selector remain UNBOUND. This is not an approved enum conversion rule or audible-effect specification.
