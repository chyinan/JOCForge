// pattern: Functional Core
//! Same-source F06ND, admitted profile metadata and selected F12 BSI input binding.

use std::sync::Arc;

use jocforge_domain::{
    analysis::source_frame::{SharedAnalysisIdentity, SharedAnalysisReceipt},
    codec::{
        frame_builder::Flat7xD0MixingMetadataValues,
        frame_syntax::{SelectedBsiDrcSyntaxError, bind_selected_bsi_drc_fields},
        selected_frame_trial::{
            SelectedBsiInformationInput, SelectedBsiInput, SelectedBsiMixingInput,
            SelectedFramePlanError,
        },
    },
    drc::NoneDrcOwnerAfterProposal,
    frontend::job_configuration::{SelectedDrcProfile, SelectedHeightTrim, SelectedSurroundTrim},
    meter::prepared_job_identity::PreparedJobInputIdentity,
    profile::{EncodingProfile, resolve_selected_profile},
    timeline::{CodecSourceIndex, source_for_callback},
    transaction::{AttemptId, CallbackLineageIdentity, ProgrammeId},
};

use crate::PreparedJobInput;

/// Failures while binding the admitted selected profile and same-source F06ND result to F12 BSI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodecSourceBridgeError {
    UnsupportedSelectedProfile,
    UnsupportedDrcEffect {
        line_profile: SelectedDrcProfile,
        rf_profile: SelectedDrcProfile,
    },
    UnsupportedTrimEffect {
        surround_trim: SelectedSurroundTrim,
        height_trim: SelectedHeightTrim,
    },
    UnsupportedSelectedMixing,
    UnsupportedSelectedDbmd,
    PreparedJobInputIdentityUnbound,
    PreparedJobInputIdentityMismatch,
    ProgrammeIdentityMismatch,
    AttemptIdentityMismatch,
    SourceIdentityMismatch,
    CallbackSourceMismatch,
    DrcFields(SelectedBsiDrcSyntaxError),
    SelectedBsi(SelectedFramePlanError),
}

pub const FLAT7X_D0_BSI_SOURCE_POLICY_ID: &str =
    "JOCFORGE_FLAT7X_D0_BSI_SAME_SOURCE_F06ND_AUTHORED_MIX_V1";

/// Same-source F06ND DRC values and admitted authored mixing values for one Flat7x D0 frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flat7xD0BsiSourceInputs {
    source_identity: SharedAnalysisIdentity,
    dialnorm: u8,
    compr: Option<u8>,
    dynamic_range_codes: [Option<u8>; 6],
    mixing: Flat7xD0MixingMetadataValues,
}

impl Flat7xD0BsiSourceInputs {
    pub fn source_identity(&self) -> &SharedAnalysisIdentity {
        &self.source_identity
    }

    pub const fn dialnorm(&self) -> u8 {
        self.dialnorm
    }

    pub const fn compr(&self) -> Option<u8> {
        self.compr
    }

    pub const fn dynamic_range_codes(&self) -> [Option<u8>; 6] {
        self.dynamic_range_codes
    }

    pub const fn mixing(&self) -> Flat7xD0MixingMetadataValues {
        self.mixing
    }

    pub const fn policy_id(&self) -> &'static str {
        FLAT7X_D0_BSI_SOURCE_POLICY_ID
    }

    pub const fn production_allowed(&self) -> bool {
        false
    }
}

/// Binds profile metadata and the exact same-source DRC proposal into the selected F12 BSI.
///
/// The application input, Analysis source transaction and DRC measurement binding must carry
/// one shared `PreparedJobInputIdentity` Arc created at admission. Unbound test fixtures remain
/// usable by their leaf tests, but cannot enter this bridge.
pub fn bind_selected_codec_bsi_source(
    prepared: &PreparedJobInput,
    shared_analysis: &SharedAnalysisReceipt,
    drc_after: &NoneDrcOwnerAfterProposal,
) -> Result<SelectedBsiInput, CodecSourceBridgeError> {
    validate_source_identity(prepared, shared_analysis, drc_after)?;
    let (mixing, information) = selected_bsi_profile_inputs(prepared)?;
    let drc_fields = bind_selected_bsi_drc_fields(shared_analysis, drc_after)
        .map_err(CodecSourceBridgeError::DrcFields)?;
    SelectedBsiInput::from_drc_fields(shared_analysis, &drc_fields, mixing, information)
        .map_err(CodecSourceBridgeError::SelectedBsi)
}

/// Binds target D0 BSI controls to the exact same-source F06ND proposal and admitted downmix.
///
/// The mapping reuses the existing selected F06ND-to-F12 field projection. It carries numeric RF
/// and per-block line codes into the target D0 structure and keeps the authored downmix values
/// separate from PCM row roles. Other optional D0 information/addbsi fields remain caller policy.
pub fn bind_flat7x_d0_bsi_source_inputs(
    prepared: &PreparedJobInput,
    shared_analysis: &SharedAnalysisReceipt,
    drc_after: &NoneDrcOwnerAfterProposal,
) -> Result<Flat7xD0BsiSourceInputs, CodecSourceBridgeError> {
    validate_source_identity(prepared, shared_analysis, drc_after)?;
    let drc_fields = bind_selected_bsi_drc_fields(shared_analysis, drc_after)
        .map_err(CodecSourceBridgeError::DrcFields)?;
    let (mixing, _) = selected_bsi_profile_inputs(prepared)?;
    Ok(Flat7xD0BsiSourceInputs {
        source_identity: shared_analysis.identity().clone(),
        dialnorm: drc_fields.dialnorm(),
        compr: drc_fields.rf_code(),
        dynamic_range_codes: *drc_fields.line_codes(),
        mixing: map_flat7x_d0_mixing_values(mixing),
    })
}

fn map_flat7x_d0_mixing_values(values: SelectedBsiMixingInput) -> Flat7xD0MixingMetadataValues {
    Flat7xD0MixingMetadataValues {
        mixmdo: values.preferred_downmix_mode,
        ltrtcmixlev: values.ltrt_center_mix_level,
        lorocmixlev: values.loro_center_mix_level,
        ltrtsurmixlev: values.ltrt_surround_mix_level,
        lorosurmixlev: values.loro_surround_mix_level,
    }
}

fn validate_source_identity(
    prepared: &PreparedJobInput,
    shared_analysis: &SharedAnalysisReceipt,
    drc_after: &NoneDrcOwnerAfterProposal,
) -> Result<(), CodecSourceBridgeError> {
    let prepared_identity = prepared.domain_identity_capability();
    let source_identity = shared_analysis.identity();
    if drc_after.shared_analysis_identity() != source_identity {
        return Err(CodecSourceBridgeError::SourceIdentityMismatch);
    }
    let source_stage_key = source_identity.source_stage_key();
    SourceIdentityContext {
        prepared_input_identity: &prepared_identity,
        shared_prepared_input_identity: source_identity.prepared_job_input_identity(),
        drc_prepared_input_identity: drc_after.prepared_job_input_identity(),
        prepared_programme_id: prepared.programme_id(),
        drc_programme_id: drc_after.programme_id(),
        callback_identity: drc_after.callback_identity(),
        drc_attempt: drc_after.attempt(),
        shared_attempt: source_stage_key.attempt_id(),
        shared_source_index: source_identity.source_index(),
        drc_source_index: drc_after.source_index(),
    }
    .validate()
}

struct SourceIdentityContext<'a> {
    prepared_input_identity: &'a Arc<PreparedJobInputIdentity>,
    shared_prepared_input_identity: Option<&'a Arc<PreparedJobInputIdentity>>,
    drc_prepared_input_identity: Option<&'a Arc<PreparedJobInputIdentity>>,
    prepared_programme_id: &'a ProgrammeId,
    drc_programme_id: &'a ProgrammeId,
    callback_identity: &'a CallbackLineageIdentity,
    drc_attempt: AttemptId,
    shared_attempt: AttemptId,
    shared_source_index: CodecSourceIndex,
    drc_source_index: CodecSourceIndex,
}

impl SourceIdentityContext<'_> {
    fn validate(&self) -> Result<(), CodecSourceBridgeError> {
        let Some(shared_identity) = self.shared_prepared_input_identity else {
            return Err(CodecSourceBridgeError::PreparedJobInputIdentityUnbound);
        };
        let Some(drc_identity) = self.drc_prepared_input_identity else {
            return Err(CodecSourceBridgeError::PreparedJobInputIdentityUnbound);
        };
        if !Arc::ptr_eq(self.prepared_input_identity, shared_identity)
            || !Arc::ptr_eq(self.prepared_input_identity, drc_identity)
        {
            return Err(CodecSourceBridgeError::PreparedJobInputIdentityMismatch);
        }

        if self.prepared_programme_id != self.drc_programme_id
            || self.prepared_programme_id != self.callback_identity.programme_id()
        {
            return Err(CodecSourceBridgeError::ProgrammeIdentityMismatch);
        }
        if self.drc_attempt != self.callback_identity.attempt_id()
            || self.drc_attempt != self.shared_attempt
        {
            return Err(CodecSourceBridgeError::AttemptIdentityMismatch);
        }
        if self.drc_source_index != self.shared_source_index {
            return Err(CodecSourceBridgeError::SourceIdentityMismatch);
        }
        if source_for_callback(self.callback_identity.callback_index())
            != Some(self.shared_source_index)
        {
            return Err(CodecSourceBridgeError::CallbackSourceMismatch);
        }
        Ok(())
    }
}

fn selected_bsi_profile_inputs(
    prepared: &PreparedJobInput,
) -> Result<(SelectedBsiMixingInput, SelectedBsiInformationInput), CodecSourceBridgeError> {
    let configuration = prepared.configuration();
    let selected_profile = configuration.selected_profile();
    let selected_profile_definition = EncodingProfile::selected_with_rate_and_carriage(
        selected_profile.codec_rate(),
        selected_profile.carriage(),
    );
    let resolved_profile = resolve_selected_profile(&selected_profile_definition)
        .map_err(|_| CodecSourceBridgeError::UnsupportedSelectedProfile)?;
    let line_drc_profile = configuration.line_drc_profile();
    let rf_drc_profile = configuration.rf_drc_profile();
    if !matches!(
        (line_drc_profile, rf_drc_profile),
        (SelectedDrcProfile::None, SelectedDrcProfile::None)
            | (SelectedDrcProfile::FilmLight, SelectedDrcProfile::FilmLight)
    ) {
        return Err(CodecSourceBridgeError::UnsupportedDrcEffect {
            line_profile: line_drc_profile,
            rf_profile: rf_drc_profile,
        });
    }
    let surround_trim = configuration.surround_trim_5_1();
    let height_trim = configuration.height_trim_5_1();
    if surround_trim != SelectedSurroundTrim::Auto || height_trim != SelectedHeightTrim::Auto {
        return Err(CodecSourceBridgeError::UnsupportedTrimEffect {
            surround_trim,
            height_trim,
        });
    }
    let wave = configuration.wave();
    // This joins input-side WAVE channels to admitted CHNA rows only; it does not
    // populate a DDP per-call frame-view channel count.
    let physical_channel_count = usize::from(wave.channels());
    let admitted_track_count = prepared.graph().graph().tracks().len();
    if resolved_profile != selected_profile
        || !configuration.input_kind().is_adm_bwf()
        || wave.sample_rate_hz() != 48_000
        || physical_channel_count == 0
        || physical_channel_count > usize::from(u8::MAX) + 1
        || physical_channel_count != admitted_track_count
        || wave.pcm_bits() != 24
        || selected_profile.codec_rate().kbps() != Some(configuration.bit_rate_kbps())
        || selected_profile.codec_rate().selector_code() != Some(configuration.bit_rate_enum())
        // F01E's selected layout enum is 8; the resolved E-AC-3 profile carries acmod=7.
        || configuration.layout_enum() != 8
        || configuration.target_capacity() != 15
        || configuration.sample_length_enum() != 1
        || configuration.frame_samples() != 1_536
        || configuration.spatial_config_mode() != 0
        || configuration.dispatch_mode() != 2
        || configuration.route_virtual_prefix_count() != 11
        || configuration.source_samples() == 0
    {
        return Err(CodecSourceBridgeError::UnsupportedSelectedProfile);
    }

    let downmix = configuration.downmix();
    if downmix.preferred_mode() != "loro" || downmix.mix_levels() != [-3; 4] {
        return Err(CodecSourceBridgeError::UnsupportedSelectedMixing);
    }

    let dbmd = prepared.dbmd();
    let surround_ex_mode = if dbmd.authored_source_mode_byte() == 3 {
        2
    } else {
        0
    };
    if dbmd.surround_ex_mode() != surround_ex_mode {
        return Err(CodecSourceBridgeError::UnsupportedSelectedDbmd);
    }

    // F06.4 maps selected Loro mode and each admitted -3 dB level to BSI codes 2 and 4.
    // F12 S.7 fixes both metadata-presence bits to one for this selected profile.
    let mixing = SelectedBsiMixingInput {
        preferred_downmix_mode: 2,
        ltrt_center_mix_level: 4,
        loro_center_mix_level: 4,
        ltrt_surround_mix_level: 4,
        loro_surround_mix_level: 4,
    };
    // F06.4 supplies these selected information values; the sample-rate code is false at 48 kHz.
    let information = SelectedBsiInformationInput {
        bitstream_mode: 0,
        copyright: true,
        original_bitstream: true,
        surround_ex_mode,
        source_sample_rate_code: 0,
    };
    Ok((mixing, information))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SelectedAdmResourcePayloads, SelectedWaveDescriptor, prepare_selected_job_input};
    use jocforge_domain::frontend::dbmd_sections::SelectedDbmdSections;
    use jocforge_domain::frontend::job_configuration::{
        SelectedDrcProfile, SelectedHeightTrim, SelectedInputKind, SelectedSurroundTrim,
    };
    use jocforge_domain::meter::prepared_job_identity::{
        NormalizedJobConfigurationIdentity, PreparedAdmChunkSpan, PreparedAdmGraphProvenance,
        PreparedInputResourceIdentity,
    };
    use jocforge_domain::timeline::FrontendCallbackIndex;
    use serde_json::Value;
    use std::ops::Range;

    const AXML: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.xml"
    );
    const CHNA: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_track_associations.bin"
    );
    const DBMD: &[u8] = include_bytes!(
        "../../jocforge-domain/tests/fixtures/selected_adm_graph/source_metadata.dbmd"
    );
    const CHNA_ROWS: usize = 38;
    const CHNA_RECORD_BYTES: usize = 40;

    fn selected_prepared_input() -> PreparedJobInput {
        selected_prepared_input_with_channels(38)
    }

    fn selected_prepared_input_with_channels(channels: u16) -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(include_bytes!(
            "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
        ))
        .unwrap();
        let case = &vectors["config_cases"][0];
        selected_prepared_input_from_xml(channels, case["input_xml"].as_str().unwrap())
    }

    fn selected_prepared_input_with_drc_profiles(
        line_profile: &str,
        rf_profile: &str,
    ) -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(include_bytes!(
            "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
        ))
        .unwrap();
        let case = &vectors["config_cases"][0];
        let xml = case["input_xml"]
            .as_str()
            .unwrap()
            .replace(
                "<line_mode_drc_profile>none</line_mode_drc_profile>",
                &format!("<line_mode_drc_profile>{line_profile}</line_mode_drc_profile>"),
            )
            .replace(
                "<rf_mode_drc_profile>none</rf_mode_drc_profile>",
                &format!("<rf_mode_drc_profile>{rf_profile}</rf_mode_drc_profile>"),
            );
        selected_prepared_input_from_xml(38, &xml)
    }

    fn selected_prepared_input_from_xml(channels: u16, input_xml: &str) -> PreparedJobInput {
        let vectors: Value = serde_json::from_slice(include_bytes!(
            "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
        ))
        .unwrap();
        let case = &vectors["config_cases"][0];
        let descriptor = &case["pcm_descriptor"];
        let (resource, spans) = selected_resource(channels);
        let metadata = SelectedAdmResourcePayloads::new(
            &resource[spans[0].clone()],
            &resource[spans[1].clone()],
            &resource[spans[2].clone()],
        );
        prepare_selected_job_input(
            ProgrammeId::new("codec-source-bridge-test").unwrap(),
            input_xml.as_bytes(),
            SelectedWaveDescriptor::new(
                u32::try_from(descriptor["sample_rate"].as_u64().unwrap()).unwrap(),
                channels,
                u16::try_from(descriptor["pcm_bits"].as_u64().unwrap()).unwrap(),
            ),
            1,
            Arc::clone(&resource),
            metadata,
        )
        .unwrap()
    }

    fn selected_resource(channels: u16) -> (Arc<[u8]>, [Range<usize>; 3]) {
        assert!(usize::from(channels) <= CHNA_ROWS);
        let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
        append_chunk(&mut bytes, *b"fmt ", &selected_format_payload(channels));
        append_chunk(&mut bytes, *b"data", &vec![0_u8; usize::from(channels) * 3]);
        let axml = append_chunk(&mut bytes, *b"axml", AXML);
        let selected_chna = if usize::from(channels) == CHNA_ROWS {
            CHNA.to_vec()
        } else {
            let chna_length = 4 + usize::from(channels) * CHNA_RECORD_BYTES;
            let mut truncated = CHNA[..chna_length].to_vec();
            truncated[..2].copy_from_slice(&channels.to_le_bytes());
            truncated[2..4].copy_from_slice(&channels.to_le_bytes());
            truncated
        };
        let chna = append_chunk(&mut bytes, *b"chna", &selected_chna);
        let dbmd = append_chunk(&mut bytes, *b"dbmd", DBMD);
        let riff_size = u32::try_from(bytes.len() - 8).unwrap();
        bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
        (Arc::from(bytes.into_boxed_slice()), [axml, chna, dbmd])
    }

    fn selected_format_payload(channels: u16) -> Vec<u8> {
        let block_align = channels * 3;
        let mut format = Vec::with_capacity(16);
        format.extend_from_slice(&1_u16.to_le_bytes());
        format.extend_from_slice(&channels.to_le_bytes());
        format.extend_from_slice(&48_000_u32.to_le_bytes());
        format.extend_from_slice(&(48_000_u32 * u32::from(block_align)).to_le_bytes());
        format.extend_from_slice(&block_align.to_le_bytes());
        format.extend_from_slice(&24_u16.to_le_bytes());
        format
    }

    fn append_chunk(bytes: &mut Vec<u8>, chunk_id: [u8; 4], payload: &[u8]) -> Range<usize> {
        bytes.extend_from_slice(&chunk_id);
        bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
        let start = bytes.len();
        bytes.extend_from_slice(payload);
        let end = bytes.len();
        if payload.len() % 2 != 0 {
            bytes.push(0);
        }
        start..end
    }

    fn identity_capability(resource_word: u8, config_word: u8) -> Arc<PreparedJobInputIdentity> {
        let resource = PreparedInputResourceIdentity::new([resource_word; 32], 1_024);
        let graph = PreparedAdmGraphProvenance::new(
            resource,
            PreparedAdmChunkSpan::new(*b"axml", 10, 10, [0x11; 32]),
            PreparedAdmChunkSpan::new(*b"chna", 20, 10, [0x22; 32]),
            PreparedAdmChunkSpan::new(*b"dbmd", 30, 10, [0x33; 32]),
        );
        let configuration = NormalizedJobConfigurationIdentity::new(
            [config_word; 32],
            [0x44; 32],
            SelectedInputKind::Adm,
        );
        Arc::new(PreparedJobInputIdentity::new(resource, configuration, graph).unwrap())
    }

    #[test]
    fn selected_bsi_profile_fields_follow_validated_configuration_and_dbmd() {
        let prepared = selected_prepared_input();
        assert_eq!(prepared.configuration().layout_enum(), 8);
        let identity = prepared.domain_identity_capability();
        assert!(Arc::ptr_eq(
            &identity,
            &prepared.domain_identity_capability()
        ));
        let (mixing, information) = selected_bsi_profile_inputs(&prepared).unwrap();
        assert_eq!(mixing.preferred_downmix_mode, 2);
        assert_eq!(mixing.ltrt_center_mix_level, 4);
        assert_eq!(mixing.loro_center_mix_level, 4);
        assert_eq!(mixing.ltrt_surround_mix_level, 4);
        assert_eq!(mixing.loro_surround_mix_level, 4);
        assert_eq!(information.bitstream_mode, 0);
        assert!(information.copyright);
        assert!(information.original_bitstream);
        let dbmd: &SelectedDbmdSections = prepared.dbmd();
        let expected_surround_ex = if dbmd.authored_source_mode_byte() == 3 {
            2
        } else {
            0
        };
        assert_eq!(information.surround_ex_mode, expected_surround_ex);
        assert_eq!(dbmd.surround_ex_mode(), expected_surround_ex);
        assert_eq!(information.source_sample_rate_code, 0);
    }

    #[test]
    fn flat7x_d0_mixing_fields_project_the_admitted_authored_levels() {
        let selected = SelectedBsiMixingInput {
            preferred_downmix_mode: 2,
            ltrt_center_mix_level: 4,
            loro_center_mix_level: 4,
            ltrt_surround_mix_level: 4,
            loro_surround_mix_level: 4,
        };
        assert_eq!(
            super::map_flat7x_d0_mixing_values(selected),
            jocforge_domain::codec::frame_builder::Flat7xD0MixingMetadataValues {
                mixmdo: 2,
                ltrtcmixlev: 4,
                lorocmixlev: 4,
                ltrtsurmixlev: 4,
                lorosurmixlev: 4,
            }
        );
    }

    #[test]
    fn previously_admitted_film_light_drc_pair_still_reaches_codec_inputs() {
        let prepared = selected_prepared_input_with_drc_profiles("film_light", "film_light");
        assert_eq!(
            prepared.configuration().line_drc_profile(),
            SelectedDrcProfile::FilmLight
        );
        assert_eq!(
            prepared.configuration().rf_drc_profile(),
            SelectedDrcProfile::FilmLight
        );
        assert!(selected_bsi_profile_inputs(&prepared).is_ok());
    }

    #[test]
    fn admitted_music_profiles_are_preserved_but_dynamic_effect_is_gated() {
        for line_name in ["music_standard", "music_light"] {
            let prepared = selected_prepared_input_with_drc_profiles(line_name, "film_light");
            let line_profile = SelectedDrcProfile::from_xml_value(line_name).unwrap();
            assert_eq!(prepared.configuration().line_drc_profile(), line_profile);
            assert_eq!(
                prepared.configuration().rf_drc_profile(),
                SelectedDrcProfile::FilmLight
            );
            assert_eq!(
                selected_bsi_profile_inputs(&prepared).unwrap_err(),
                CodecSourceBridgeError::UnsupportedDrcEffect {
                    line_profile,
                    rf_profile: SelectedDrcProfile::FilmLight,
                }
            );
        }
    }

    #[test]
    fn non_auto_trim_values_are_preserved_but_not_silently_ignored() {
        let vectors: Value = serde_json::from_slice(include_bytes!(
            "../../jocforge-domain/tests/fixtures/job_configuration/job-configuration-vectors.json"
        ))
        .unwrap();
        let base_xml = vectors["config_cases"][0]["input_xml"].as_str().unwrap();
        let xml = base_xml.replace(
            "<surround_trim_5_1>auto</surround_trim_5_1>",
            "<surround_trim_5_1>-3</surround_trim_5_1>",
        );
        let prepared = selected_prepared_input_from_xml(38, &xml);
        assert_eq!(
            prepared.configuration().surround_trim_5_1(),
            SelectedSurroundTrim::Minus3Db
        );
        assert_eq!(
            prepared.configuration().height_trim_5_1(),
            SelectedHeightTrim::Auto
        );
        assert_eq!(
            selected_bsi_profile_inputs(&prepared).unwrap_err(),
            CodecSourceBridgeError::UnsupportedTrimEffect {
                surround_trim: SelectedSurroundTrim::Minus3Db,
                height_trim: SelectedHeightTrim::Auto,
            }
        );
    }

    #[test]
    fn selected_bsi_profile_fields_accept_variable_adm_rows_without_frame_view_join() {
        let prepared = selected_prepared_input_with_channels(36);
        assert_eq!(prepared.graph().graph().tracks().len(), 36);
        let (mixing, information) = selected_bsi_profile_inputs(&prepared).unwrap();
        assert_eq!(mixing.preferred_downmix_mode, 2);
        assert_eq!(mixing.loro_center_mix_level, 4);
        assert_eq!(information.source_sample_rate_code, 0);
        assert_eq!(
            information.surround_ex_mode,
            prepared.dbmd().surround_ex_mode()
        );
    }

    #[test]
    fn same_programme_attempt_rejects_different_resource_capability() {
        let prepared = identity_capability(0x11, 0x33);
        let different_resource = identity_capability(0x22, 0x33);
        assert_ne!(
            prepared.resource().input_sha256(),
            different_resource.resource().input_sha256()
        );
        assert_eq!(
            prepared.normalized_configuration().config_sha256(),
            different_resource
                .normalized_configuration()
                .config_sha256()
        );
        let programme = ProgrammeId::new("same-programme").unwrap();
        let attempt = AttemptId::new(7);
        let callback =
            CallbackLineageIdentity::new(programme.clone(), attempt, FrontendCallbackIndex::new(2));
        assert_eq!(programme, callback.programme_id().clone());
        assert_eq!(attempt, callback.attempt_id());
        assert_eq!(
            SourceIdentityContext {
                prepared_input_identity: &prepared,
                shared_prepared_input_identity: Some(&prepared),
                drc_prepared_input_identity: Some(&different_resource),
                prepared_programme_id: &programme,
                drc_programme_id: &programme,
                callback_identity: &callback,
                drc_attempt: attempt,
                shared_attempt: attempt,
                shared_source_index: CodecSourceIndex::new(0),
                drc_source_index: CodecSourceIndex::new(0),
            }
            .validate(),
            Err(CodecSourceBridgeError::PreparedJobInputIdentityMismatch)
        );
    }

    #[test]
    fn same_programme_attempt_rejects_different_configuration_capability() {
        let prepared = identity_capability(0x11, 0x33);
        let different_configuration = identity_capability(0x11, 0x55);
        assert_eq!(
            prepared.resource().input_sha256(),
            different_configuration.resource().input_sha256()
        );
        assert_ne!(
            prepared.normalized_configuration().config_sha256(),
            different_configuration
                .normalized_configuration()
                .config_sha256()
        );
        let programme = ProgrammeId::new("same-programme").unwrap();
        let attempt = AttemptId::new(7);
        let callback =
            CallbackLineageIdentity::new(programme.clone(), attempt, FrontendCallbackIndex::new(2));
        assert_eq!(programme, callback.programme_id().clone());
        assert_eq!(attempt, callback.attempt_id());
        assert_eq!(
            SourceIdentityContext {
                prepared_input_identity: &prepared,
                shared_prepared_input_identity: Some(&prepared),
                drc_prepared_input_identity: Some(&different_configuration),
                prepared_programme_id: &programme,
                drc_programme_id: &programme,
                callback_identity: &callback,
                drc_attempt: attempt,
                shared_attempt: attempt,
                shared_source_index: CodecSourceIndex::new(0),
                drc_source_index: CodecSourceIndex::new(0),
            }
            .validate(),
            Err(CodecSourceBridgeError::PreparedJobInputIdentityMismatch)
        );
    }

    #[test]
    fn unbound_test_fixture_identity_is_not_admitted_to_the_production_bridge() {
        let prepared = identity_capability(0x11, 0x33);
        let programme = ProgrammeId::new("same-programme").unwrap();
        let attempt = AttemptId::new(7);
        let callback =
            CallbackLineageIdentity::new(programme.clone(), attempt, FrontendCallbackIndex::new(2));
        assert_eq!(
            SourceIdentityContext {
                prepared_input_identity: &prepared,
                shared_prepared_input_identity: None,
                drc_prepared_input_identity: None,
                prepared_programme_id: &programme,
                drc_programme_id: &programme,
                callback_identity: &callback,
                drc_attempt: attempt,
                shared_attempt: attempt,
                shared_source_index: CodecSourceIndex::new(0),
                drc_source_index: CodecSourceIndex::new(0),
            }
            .validate(),
            Err(CodecSourceBridgeError::PreparedJobInputIdentityUnbound)
        );
    }
}
