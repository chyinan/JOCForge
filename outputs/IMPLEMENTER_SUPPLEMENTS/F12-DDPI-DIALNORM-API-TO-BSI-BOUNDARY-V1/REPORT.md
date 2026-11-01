# DDPi DialNorm API-to-BSI boundary

## Static API contract

For the hash-identified `dee_ddp_encoder.exe`, DDPi parameter ID `0x12` is labeled DialNorm in the type table for entries 0, 1, 3, and 4. Its accepted integer range is 1–31; invalid values map to “Invalid value for DialNorm.” Which type entry is active was not observed. The setter copies the 32-bit value unchanged to per-stream state `+0x9C`; the validator rejects values outside the range. There is no scaling or rounding in that setter/validation path.

## Writer boundary

The traced frame-processing path reaches the DDP stuffer. However, the validated field is `stream+0x9C`, while the first converter is passed `stream+0x0C` and reads relative `+0x9C`, i.e. `stream+0xA8`. No direct read of the validated `+0x9C` field or its mapping to the standard header DialNorm U5 was found along this bounded chain. The last direct consumer is the range validator; `FUN_140317600` is the next function to inspect.

This does not contradict the washed F12 model or the multi-input conditional matches: it narrows the native writer boundary. The model’s assumption that DEE's unitless scalar is `selected_lufs` remains conditional, and no native scalar-to-`0x12` or serialized-U5 join is proven.

## Disposition

Reference only; `implementation_ready=false`. Keep the parameter API integer range 1–31 within its descriptor scope, but do not equate that value with final U5 or implement a conversion from measured scalar. No DEE run, PCM, payload, bitstream, output, key, or license material was read.
