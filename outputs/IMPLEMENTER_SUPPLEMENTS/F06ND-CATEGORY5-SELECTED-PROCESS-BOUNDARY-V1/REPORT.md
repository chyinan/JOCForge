# F06ND category5 selected-process boundary

## Result

In the hash-bound `dee_audio_filter_ddp_atmos.dll` image, the selected Atmos processor path receives frame/channel data and has no category5 or `InputControlEvent` parameter on the inspected process interface. The nonzero-frame branch copies per-channel samples into an internal `MultiChannelBuffer`, calls the Atmos backend, advances sample cursors, and prepares an output descriptor. No category5 byte record is written in this selected process path; its byte layout is `NOT_APPLICABLE_TO_THIS_PROCESS_PATH`.

The same image exposes a separate generic `AdmFileStorage::writeData<int/float>` API accepting `vector<InputControlEvent>`. The Wash evidence does not join the selected category5 producer to that vector or to a BSI/output consumer. Keep the generic storage-event edge and any broader category5-to-output effect `UNBOUND`.

## Selected-profile rule

The independently hash-checked approved F06ND source leaf states that category5, 32-bit, count1 does not write the DRC-control FIFO for the selected profile. The four logical records remain zero and the override remains 0. The category number is not EMDF ID1 or ID11. This is an approved logical contract plus static native boundary evidence; no native run or PCM observation was made.

The approved source leaf SHA-256 is `62dace058d92eaf8c4b9262c40d885d37801e55798d0be331e07bae0117a7081`. The original `outputs/IMPLEMENTER_PACKAGE/...` path named in the evidence package was absent in this checkout; the byte-identical approved leaf was found in the implementation-side `F06NR-APPROVED-SOURCE-LEAVES-V1` package and hashed directly.

## Scope

This does not reconstruct the generic `InputControlEvent` byte schema, identify an event producer, prove category5 effects outside the selected `AtmosProcessor::process` path, or close a BSI/output join. The evidence package is `SEALED_PARTIAL`; no PCM, payload, key, license, or native execution was included.
