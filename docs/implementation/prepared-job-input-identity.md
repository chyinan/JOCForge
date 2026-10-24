# Prepared job input identity

`PreparedJobInput` now carries three linked identities:

- `ResourceIdentity` hashes the complete caller-owned RIFF byte allocation and records its byte length.
- `job_xml_sha256` hashes the raw admitted UTF-8 job document.
- `config_sha256` hashes the canonical identity envelope with the XML digest, selected input label, and complete normalized F01E configuration.

The input adapter still owns RIFF validation. `ParsedRiffPcm24::selected_adm_payloads()` returns unique borrowed AXML, CHNA, and DBMD payload spans from one parsed RIFF resource. `prepare_selected_job_input` receives those views with the same immutable allocation. It checks that each view is a subslice of that allocation and that the preceding RIFF chunk header has the matching ID and payload length. It parses the graph and DBMD from those same views, then stores their offsets, lengths, payload digests, and parent `ResourceIdentity` together.

The dynamic primary-control binding carries that same resource identity, data extent, metadata hashes, job digest, and config digest into the selected installed48 source-map builder. The detached historical projection path retains its own admission checks; the selected ADM path uses the same row mapping after its admitted runtime identity checks.

## Internal validation run

The approved internal validation WAV was read in memory from the existing package. Its observed complete resource was 743,696,526 bytes with SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. A local F01E caller document based on the accepted configuration vector used the selected `input-adm.wav` reference; this was a constructed validation job, not the source's original job XML.

The run admitted the same-resource AXML/CHNA/DBMD graph, 38 physical tracks, 28 object events, and 38 DBMD track rows. It staged the first actual PCM source frame and primary activity values in memory. The F03R typed-control binding remains unresolved: the ADM event `control52` diffuse word is not established as the F03R feature-flags diffuse input. The probe therefore discarded the callback proposal and reported no completed F03R, feature, selector, component, OAR packet, meter result, or receipt.

No PCM was written to disk. Whole-song encoding, EOF/drain behavior, and product acceptance remain unrun.
