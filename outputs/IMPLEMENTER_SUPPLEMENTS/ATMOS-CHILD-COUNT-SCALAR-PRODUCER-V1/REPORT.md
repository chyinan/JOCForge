# Atmos child-count scalar producer

## Static producer/getter chain

In the ordinary count path, `FUN_18054fa70` calls the child vtable `+0x80`
getter. For `atmos_storage::MasterSourceImpl`, that getter is
`FUN_180536320`; it returns the U16 at `*(child+0x28)+0x0A` and does not
directly read a vector. `FUN_180536180` stores its third constructor argument
at `child+0x28`.

The ADM BWF type-3 open path `FUN_1804d43c0` returns a config pointer at
`object+0x40` into the child record and writes the U16 at `object+0x4A`
(`config+0x0A`) from `FUN_1802816c0`. That producer adds the Object source
count from storage vtable `+0xC0` to the sum of bed-row channel-vector
lengths. It obtains bed-row count from `+0xB8`, each row through `+0xC8`,
and derives row length as `(end@+0x50 - begin@+0x48)/16`. The opener then
allocates six-byte channel-map records from the resulting scalar.

## Sample metadata candidate and boundary

For the washed `人是猫` metadata, the ten DirectSpeakers rows have one AXML
channel reference each, summing to 10; there are 28 Object tracks. This gives
`28+10=38` as a metadata-conditioned candidate. It does not prove that the
native storage interface had these exact row vectors or that the runtime
getter returned 38.

The composite count routine also has a separate special-mode formula
`10 + max(marked object records)`, selected by a flag or mismatch. The mode
chosen by this run, storage row identity, runtime U16 and full output channel
mapping remain `UNBOUND/NO`.

## Scope and exclusions

Ghidra 12.2 static evidence only, combined with already washed metadata. No
WAV/BWF or PCM was opened, no DEE run occurred, and no key/license material
was accessed. This supplement contains no binaries, raw decompilation,
media, PCM or encoded payload.
