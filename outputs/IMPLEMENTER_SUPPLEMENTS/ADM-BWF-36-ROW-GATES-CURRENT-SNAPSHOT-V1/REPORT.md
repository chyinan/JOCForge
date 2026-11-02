# Current 36-row admission and downstream gates

**Status:** `JOB_XML_PARSE_RANGE_DYNAMIC; PRIMARY_CONTROL_AND_FRESH_ZERO_STILL_38; FULL_PATH_NOT_RUN`

The seven authorized ADM metadata scan contains six 36-row files and one 38-row file. All are RIFF/WAVE, PCM24, 48 kHz, with ten DirectSpeakers bed rows and 26 or 28 Objects rows. The scan skipped every `data` payload.

In the current JOCForge working tree, `parse_selected_job_configuration` now accepts 1–256 channels for both internal `<adm>` and `<atmos_mezz>` variants. This edit is uncommitted and the current `job_configuration.rs` SHA-256 is `a22a78e30a0c149f1eda42f76cdf86ba0dc7f0a33f674b59a3f2c8c18be22b40`. The selected CLI request emits `<adm>`.

That parser change does not remove downstream 38-row requirements. `bind_admitted_selected_input` still checks `channels == SELECTED_RUNTIME_CHANNELS` (38), and the normalized identity still includes the label `selected_adm_bwf_38ch_pcm24_48khz`. `normal_control_snapshot` remains 38 physical / 10 bed / 28 object rows. `validate_fresh_zero_callback_identity` still requires 38 physical rows at line 1418. These checks concern distinct receipt/control/runtime layers; this package does not determine which can safely become dynamic.

The 36-row PreparedJobInput test fixture exists in source, but the seven-file scan did not run it. There is still no 36-row full meter, JOC, or EC3 output receipt. The next bound algorithmic gate is the fresh-zero/F01E owner reset path, which must retain the DEE processing dimensions while separating authored input rows.
