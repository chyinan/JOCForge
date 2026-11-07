# Preferred downmix MixMode initial-value boundary

## Bound field and first observed write

The selected `MixMode+0x50` value field is `FUN_180136220.param_1+0x960`. A targeted exact-field inspection of this owner initializer found no instruction-text or decompiler references to `+0x960`. Reused parser evidence reports no direct `+0x50` store in `FUN_180171320` or `FUN_180171440`.

The first observed write remains the later typed setter `FUN_18014f6e0`, which stores the return from `FUN_180143cf0`. When the optional XML-constructor callback is null, that helper returns the pre-existing MixModeValue. No constant write, typed caller-argument write, or confirmed zero initialization before the setter was found in the bounded owner initializer.

## Boundary

The pre-existing value's source remains `UNBOUND`; zero initialization is `NOT_CONFIRMED`. Runtime selected value is `NOT_RUN`, and the audio/BSI consumer remains `UNBOUND`. The field bytes were not read, and the search was limited to the selected initializer rather than a displacement-wide scan.

## Provenance

Source package `DEE-ATMOS-DDP-PREFERRED-DOWNMIX-MIXMODE-INITIAL-VALUE-BOUNDARY-WASH-V1` manifest SHA-256 `6be2353eba98fc3d79247f1f61d2c5777f6c913c13ab08235b098fe208ba1125`; normal and `python -O` checks passed.
