# DDP record to composite-master reader lineage

## Scope

This addendum binds a DDP-internal typed path from per-source record construction through the AtmosReader and composite-master getter to the `+0xC8` descriptor dispatch. It also corrects the classification of DDP owner `+0x350`. It does not connect the host-selected ADM owner or its generation to this DDP path.

## DDP-internal path

`FUN_18014ABA0` builds a per-source set via `FUN_180145380`, selects an entry, and passes it as `param_3` to `FUN_1801535B0`. The builder reads DDP owner `+0xA0` and `+0x198` maps. `+0x350` is an output-file list checked against encode/source-set count; it is not the per-source array.

The 336-byte measurement record is forwarded through `FUN_180176980`, `FUN_1801769A0`, and `FUN_1801764E0` into AtmosReader setup. Composition setup `FUN_1801828C0` passes `composition_state+0x50` as the output argument to the composite-master factory. On success the constructed object is stored there. Getter `FUN_18032F1D0` returns the same field; `FUN_180186870` uses that value as the receiver of virtual dispatch at `+0xC8`, then records the DDP row table pointer and channel/frame counts in its descriptor.

Run151 observed the dispatch target as `atmos_storage_composite_master` and observed descriptor offset 0 equal to the row-table token. These observations bind the DDP-internal receiver path, not the host input owner.

## Boundaries

The host-selected ADM master/owner and generation are not joined to the DDP per-source record or composite master. Numeric stack-address equality remains non-identity. Run152/153 job-hash correction is provenance only; their input payload identity remains `UNBOUND/NO`. No new encoder run or bitstream parse occurred. Audio/JOC playback remains `NOT_RUN`.
