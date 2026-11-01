# R138 job to component mode-source boundary

## Static edge

The hash-bound R138 job declares the `audio.adm` input and `encode_to_atmos_ddp` output profile, with 1024 kbps, embedded-timecode range 86640–86664, measure-only 1770-4, DI off, and line/RF DRC none. The custom DialNorm element is absent.

The bounded Ghidra path reaches the component state used by the encode callback. Its configuration adapter reads component member `+0x54` as the source of the translated context-mode Boolean; data rate comes from a separate component member. No typed transfer from any R138 XML field to `+0x54` was found, and the owner-side setter/initializer is outside this path.

## Disposition

R138 component mode, update flag, owner U16, and applicability of the conditional custom-BSI zero rule remain `UNBOUND`. The absence of a custom DialNorm XML element does not prove the value of component member `+0x54`. The next bounded edge is the first writer/initializer of that member in the component-open/configuration path, typed back to the R138 job decoder.

Ghidra 12.2 static analysis only. No DEE run, U16 capture, PCM/payload read, or key/license access occurred. Reference-only; `implementation_ready=false`.
