# U04 helper return interpretation correction

The updated source package u04-projection-consumer-wash-v1 has MANIFEST SHA-256 4191de2a79eb4d72957ef3f5fd08b5696e67a0e5caa56294249e38f9e9d1fd04. It supersedes its previous manifest. Same-image approved helper-return disassembly establishes that the helper returns raw argument 4 (mode). Therefore the earlier RAX=0 observed under mode 0 is a raw-mode echo, not success/status. The v10 dynamic capture did not recapture RAX at return; this interpretation comes from the same-image approved static helper contract.

Do not describe the helper as void and do not treat zero as success. Call 1 output captured in v10 is all zero despite nonzero input. Call 2 first output contribution was observed at the consumer; the full 20-value reduction and source-weight mapping remain open. Exact job/config/input identities and payloads are in CURRENT-JOB-U04-PROJECTION-CONSUMER-V10. This correction changes interpretation only, not U04 closure status.
