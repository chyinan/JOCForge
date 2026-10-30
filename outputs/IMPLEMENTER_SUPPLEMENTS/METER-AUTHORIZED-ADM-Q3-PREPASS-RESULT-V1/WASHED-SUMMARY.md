# Authorized real-input Meter Q+3 run result

Disposition: LOCAL_REAL_INPUT_Q3_AND_TERMINAL_MEASUREMENT_RECEIPT_PROPOSAL_PASS; GTC READY AND PRODUCT OUTPUT NOT RUN.

The user-authorized 38-channel ADM BWF was verified before execution at 743,696,526 bytes and SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. Its selected wave extent was 48 kHz, PCM24, 38 channels, block align 114, and 6,522,906 samples. The selected job XML hash was `20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9`.

The explicitly selected integration test `authorized_adm_bwf_stages_complete_independent_meter_prepass_receipt` passed once in 3,293.53 seconds. It staged 4,247 normal callbacks plus three fresh-zero drains, collected 4,250 typed Meter owner receipts, reached the complete source extent, and produced a terminal MeasurementReceipt proposal. The measurement-dependent receipt matched the terminal receipt; ID1 bound the same prepared source digest; the DRC root and metadata cold owner were constructed. The standalone Meter root remained unchanged.

Evidence grades are limited to the actual local integration test and its assertions. The receipt was staged as a proposal; this run did not admit GTC READY or swap the twelve-owner programme root. It did not run W04, produce codec AUs, encode or publish an output, or establish playback or product acceptance. No input path, PCM, partial output, private key, source code, or decompilation material is included in this package.
