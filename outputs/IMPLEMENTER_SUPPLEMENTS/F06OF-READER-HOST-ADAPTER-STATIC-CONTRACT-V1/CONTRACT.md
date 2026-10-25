# ADM reader host-adapter static contract

Status: `READER_REGISTRATION_AND_BORROWED_ROW_ADAPTER_STATIC_CLOSED; SELECTED_SOURCE_TO_MEASUREMENT_JOIN_UNBOUND`.

## Registration and storage creation

The reviewed host initialization path registers an ADM BWF reader factory for registry type value 3 and installs two callbacks. The factory can create a master reader handle while retaining the factory reference. A later lazy-open call uses that factory to create an `AdmFileStorage` instance and stores the resulting shared storage reference in the master handle.

These are static registration and construction paths. They do not prove that the selected measured job requested registry type 3, that this factory created its handle, or that its source belongs to the selected measurement generation.

## Float-read adapter and row ownership

The master reader's float-read interface receives a caller row-pointer table, an event/result collection, requested frames per channel, a destination offset in samples, and an actual-frame output. The adapter wraps a non-null caller row table in a temporary buffer descriptor with a channel count, sample capacity of `requested + offset`, and the borrowed table pointer. The descriptor does not own the caller's row storage. The helper frees the temporary descriptor, not the caller rows. A null table uses a separate allocated-buffer path; its use in the selected run is not established.

The adapter lazily opens storage when needed and calls the storage read interface with the original requested-frame count. It does not pre-clip that request at this layer. On the normal status-zero branch, the outward actual-frame count is `min(remaining_duration_frames, reader_returned_frames)`.

The buffer descriptor's local channel field and the downstream meter descriptor's `channel_count` field occupy different record positions and belong to different interfaces. Their shapes, equal values, or row-table presence do not establish descriptor identity.

## Host dispatch and remaining join

Static host analysis shows a source property written to a plug-in object and generic stage dispatch through plug-in callbacks. Those host fields are not joined to the master reader handle, its lazily opened storage, the borrowed row table, or the meter measurement descriptor.

The unique remaining relation is a stable handoff token connecting the host source instance and its generation to the master handle/storage, the borrowed row table and actual-frame read result, and the meter descriptor's channel/frame fields in that same generation. Keep this relation `UNBOUND`; no field name, registration type, count equality, or stage order substitutes for it. EOF/drain and reset-generation behavior remain separate open claims.

This contract is static interface behavior only. It does not establish selected-run identity, whole F06OF readiness, or product acceptance.
