# Selected F01E job configuration and DBMD fields

`frontend::job_configuration::parse_selected_job_configuration` parses the selected F01E job XML, validates the 48 kHz/38-channel/PCM24 descriptor and positive source length, normalizes the fixed 1024 kbps configuration, and passes the selected row through the existing production profile resolver. Input/output paths remain references; the parser does not open files.

The parser rejects DTD/entity declarations before XML parsing. It accepts only an unnamespaced `job_config` root and the specified direct-child paths. Duplicate recognized fields reject. The selected input prefers `input/audio/adm` over `atmos_mezz`. The output node must be `output/ec3`.

Time fields follow F01E.1 exactly: optional `offset`/`ffoa` default to `auto`; `start`/`end` default to empty and accept only their selected aliases; `time_base` is `file_position`; prepend/append silence use the specified zero forms. Loudness, DRC, downmix and trim values are admitted only for the selected fixed row. Field deviations return local `CONFIG_*` errors; the resulting fixed dimensions are then checked by the existing profile resolver.

Hash-bound vectors cover 11 XML cases: three accepted configurations and eight selected rejections. Tests compare normalized config values, reference precedence, optional defaults, and exact local error categories. The XML fixture is a real selected job-config input from the approved package; it is not a synthetic replacement for an ADM graph, PCM source or codec run.

## DBMD input fields

`frontend::dbmd_sections::parse_selected_dbmd_sections` consumes a caller-supplied DBMD byte slice and parses section framing from offset 4. It checks the selected length-header collision, U16LE section lengths, per-section checksum, duplicate IDs, optional zero terminator/tail, required sections 9/10, section9 length 248, section10 magic, and selected track-table bounds. It returns warm-trim mode, authored mode byte, surround-EX mode, and per-track fields in physical track-index order. The field accessor joins by physical host index, not pack/object ordinal.

The parser is stateless and does not open RIFF files. It does not parse the F01G graph or attach track fields to ADM hosts. The job-XML and DBMD parsers are two F01E input-stage slices; they are not a complete F01E owner or a selected encoder pipeline.

Hash-bound F01E vectors cover 11 XML configurations, four authored modes, six per-track fields and ten DBMD fail-stop inputs. Expected values are checked only after the parsers produce actual results. The local fixture is `tests/fixtures/job_configuration/job-configuration-vectors.json`, pinned to the approved base package and the F01E review binding.

## Blocked integration edge

The candidate graph identity/event slice is implemented in `frontend/selected_adm_graph.rs` and `frontend/adm_object_events.rs`. It parses hash-bound candidate AXML/CHNA plus that candidate's actual raw DBMD metadata chunk, yields 38 physical rows, 28 host-ordered candidate object events, a 10-bed/28-object identity join and regular source identity 11..47, then can bind parsed DBMD fields to the control52 set by physical host. This remains a distinct secondary candidate, not the R4 primary source.

The F01E.8/.9 control-value formula is available in `frontend/object_control.rs`. Its 364-word row KATs consume the separate F01E `actual_fixture_object_controls` vector after computation; they do not claim source identity with the `hanabi.wav` candidate graph. The candidate graph now has its own AXML event→actual DBMD parser→physical-host→28-control candidate path, whose 224 control values match same-candidate downstream source-field inputs. It is still not bound to the F01E main selected job and is not native event-producer output or full frontend execution.

Wash R4 now binds a path-free primary job projection and metadata-only source receipt to primary AXML/CHNA/DBMD payload hashes and N=6,522,906 for the metadata/control producer. `frontend::job_configuration::bind_selected_primary_control_projection` validates that narrow binding; `frontend::primary_metadata_control::produce_primary_metadata_control_rows` then recomputes the 38 physical-host rows, 364 object control words, and parsed DBMD fields. Bed bytes 24..51 stay unproduced. The original job XML was not distributed, the full PCM identity is `NOT_REHASHED`, and this local value producer remains outside main `encode`.

The candidate Hanabi graph is still not used as the selected-job source. The separate approved F01E validation fixture remains labelled validation-only; neither it nor the candidate supplies PCM identity for R4. Remaining F01E blockers include callback hold/reset ownership and control38 atomic publication. Remaining F01/F01G blockers include source field0, complete skip+4 transition, prefix11 geometry/attributes integration, full installed48 numeric state, native malformed XML policy, and whole frontend audio installation. No end-to-end link is claimed.

Whole F01E/F01 and the selected ADM→JOC chain remain deferred.
