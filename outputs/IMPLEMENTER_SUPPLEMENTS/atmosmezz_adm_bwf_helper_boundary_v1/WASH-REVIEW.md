# Wash review: AtmosMezz helper ADM BWF path

Static Ghidra evidence confirms that `dee::input::AtmosMezz::Run` delegates its provider-supplied input to the hash-bound `atmos_info.exe --input` helper. That helper documents ADM BWF input, registers a BWF system, selects a type-specific master factory, constructs `atmos_storage_master_s`, and reaches the ADM BWF descriptor/ADM XML parser with `chna` and `axml` checks. The route is not restricted to a different, non-BWF representation.

This proves static helper capability, not that the R3 sample-6 run actually selected this helper factory/master. Its receipt has no helper process, selected callback, master/storage object, vtable, or `Run` target. DACT independently has a similar ADM storage class family, but no shared object or code identity is established. Keep the minimum runtime join open.

The package supplies no audio algorithm or field value. It is only an input-capability/route boundary. Audio-affecting behavior remains based on DEE observations; external parsers are posthoc checks only.
