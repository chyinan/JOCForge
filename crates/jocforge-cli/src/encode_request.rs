// pattern: Imperative Shell
//! Reads one selected ADM BWF, prepares its input identity, and enters the application encode gate.

use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::meter_prepass::{SelectedMeterPrepassError, run_selected_meter_prepass_with_progress};
use crate::output_publication::{OutputPublicationError, ProgrammeOutputStaging, finalize_output};
use crate::programme_roots::{ProgrammeOwnerRoots, source_for_callback_clock};
use crate::progress::ProgressBar;
use crate::selected_spatial_callback_providers::{
    SelectedMainSpatialProviderLoadError, load_selected_main_spatial_callback_providers_for_profile,
};
use jocforge_application::drc_source_bridge::CallbackLineageIdentity;
use jocforge_application::independent_meter_callback::CallbackTransactionId;
use jocforge_application::{
    EncodeError, JobXmlIngressError, PreparedJobInput, PreparedJobInputError, ProgrammeId,
    SelectedAdmResourcePayloads, SelectedWaveDescriptor,
    drc_source_bridge::{NoneDrcOwnerError, cold_drc_owner_after_measurement},
    independent_meter_callback::generate_selected_spatial_math_geometry,
    integrity_source_bridge::cold_integrity_owner_for_plan,
    metadata_source_bridge::{
        MetadataOwnerPreparationError, SelectedJocState, prepare_metadata_owner_from_measurement,
    },
    meter_source_bridge::{
        CodecRateProfile, EncodingProfile, Flat7xI0D0TargetPlan,
        FrontendControlReplacementCandidate, JocReconstructionLayout, ResolvedEncodingProfile,
        resolve_selected_profile,
    },
    output_publication::{AttemptId, OutputStagingOwnerId},
    parse_selected_job_file_references, prepare_selected_job_input,
    reader_eof_callback::{
        ReaderEofActualSampleCount, ReaderEofMachine, ReaderEofNormalCallbackClock,
        ReaderEofPaddingSampleCount, ReaderEofReplacementCandidate,
        stage_reader_eof_adm_input_normal_callback, stage_reader_eof_cached_drain,
    },
    source_analysis_bridge::{
        SelectedSourceAnalysisProviders, cold_analysis_root_for_prepared_input,
    },
    source_frame_bridge::CodecOwnerRoot,
    spatial_callback::{
        MainFrontendSnapshotError, MainHeldControlOutput, MainNormalControlOutput,
        MainSpatialCachedCallbackInput, MainSpatialCallbackCandidate, MainSpatialCallbackError,
        MainSpatialCallbackInput, MainSpatialColdRoots, MainSpatialColdRootsError,
        stage_main_frontend_adm_input_snapshot, stage_main_spatial_cached_callback,
        stage_main_spatial_callback,
    },
};
use jocforge_io::pcm::{
    frontend_pcm_source::{
        CodecSourceIndex, FrontendCallbackIndex, FrontendPcmSourceError, read_frontend_pcm_callback,
    },
    riff_pcm24::parse_selected_riff_pcm24,
    riff_pcm24_file::{RiffPcm24FileReadError, read_resource},
};

const ENCODE_REQUEST_STACK_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub enum EncodeRequestError {
    InvalidPath,
    JobXmlInput(std::io::Error),
    JobXmlIngress(JobXmlIngressError),
    InvalidJobXmlReference,
    UnsupportedJobXmlOption(&'static str),
    OutputAlreadyExists,
    UnsupportedDataRate,
    UnsupportedJocLayout,
    Flat7xPairNotReady,
    InputFile(RiffPcm24FileReadError),
    Riff(jocforge_io::pcm::riff_pcm24::RiffPcm24Error),
    PreparedInput(PreparedJobInputError),
    InputExtent,
    ProgrammeIdentity,
    ProductGate(EncodeError),
    OutputStaging(OutputPublicationError),
    SpatialProviders(SelectedMainSpatialProviderLoadError),
    SpatialGeometry,
    SpatialColdRoots(MainSpatialColdRootsError),
    FrontendPcm(FrontendPcmSourceError),
    FrontendSnapshot(MainFrontendSnapshotError),
    SpatialCallback(MainSpatialCallbackError),
    CallbackTransaction,
    ClockMismatch,
    WorkerUnavailable,
    WorkerPanicked,
    MeterPrepass(SelectedMeterPrepassError),
    DrcRoot(NoneDrcOwnerError),
    MetadataOwner(MetadataOwnerPreparationError),
    SourcePipeline(String),
}

impl std::fmt::Display for EncodeRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => {
                formatter.write_str("encode requires absolute input and output file paths")
            }
            Self::JobXmlInput(error) => write!(formatter, "could not read job XML: {error}"),
            Self::JobXmlIngress(error) => write!(formatter, "{error}"),
            Self::InvalidJobXmlReference => formatter.write_str(
                "job XML storage/local/path must be absolute and file_name must be a basename",
            ),
            Self::UnsupportedJobXmlOption(path) => write!(
                formatter,
                "job XML option is parsed but not supported by the selected pipeline: {path}"
            ),
            Self::OutputAlreadyExists => {
                formatter.write_str("output exists; refusing to replace it")
            }
            Self::UnsupportedDataRate => formatter.write_str(
                "data_rate must be one explicit value: 384, 448, 576, 640, 768, or 1024",
            ),
            Self::UnsupportedJocLayout => {
                formatter.write_str("joc-layout flat7x requires the selected 1024 kbps profile row")
            }
            Self::Flat7xPairNotReady => formatter.write_str(
                "Flat7x paired output is not ready: I0 audblk production, D0 source-generation binding, and pair receipts are incomplete; encode did not enter output staging",
            ),
            Self::InputFile(error) => write!(formatter, "{error}"),
            Self::Riff(error) => write!(formatter, "selected RIFF input rejected: {error:?}"),
            Self::PreparedInput(error) => write!(formatter, "selected ADM input rejected: {error}"),
            Self::InputExtent => formatter.write_str("RIFF data extent is out of range"),
            Self::ProgrammeIdentity => {
                formatter.write_str("could not create a fresh programme identity")
            }
            Self::ProductGate(error) => write!(formatter, "{error}"),
            Self::OutputStaging(error) => write!(formatter, "{error}"),
            Self::SpatialProviders(error) => write!(formatter, "{error}"),
            Self::SpatialGeometry => {
                formatter.write_str("selected P11/P15 geometry construction failed")
            }
            Self::SpatialColdRoots(error) => {
                write!(formatter, "MAIN cold owner construction failed: {error:?}")
            }
            Self::FrontendPcm(error) => {
                write!(formatter, "MAIN PCM callback read failed: {error:?}")
            }
            Self::FrontendSnapshot(error) => {
                write!(formatter, "MAIN F01E callback staging failed: {error:?}")
            }
            Self::SpatialCallback(error) => write!(
                formatter,
                "MAIN F03W/F04 callback staging failed: {error:?}"
            ),
            Self::CallbackTransaction => {
                formatter.write_str("could not create the MAIN callback transaction identity")
            }
            Self::ClockMismatch => formatter
                .write_str("selected MAIN callback g=0 has an unexpected source index or phase"),
            Self::WorkerUnavailable => {
                formatter.write_str("could not start the selected encode worker")
            }
            Self::WorkerPanicked => formatter.write_str("selected encode worker panicked"),
            Self::MeterPrepass(error) => {
                write!(formatter, "selected Meter prepass failed: {error}")
            }
            Self::DrcRoot(error) => write!(formatter, "selected DRC cold root failed: {error:?}"),
            Self::MetadataOwner(error) => {
                write!(formatter, "selected Metadata cold owner failed: {error:?}")
            }
            Self::SourcePipeline(error) => write!(formatter, "source pipeline stopped: {error}"),
        }
    }
}

impl std::error::Error for EncodeRequestError {}

/// Prepares one full-file request from the actual selected-profile RIFF resource.
///
/// The request uses file-position extent with empty start/end, so the parser's actual RIFF data
/// extent supplies `source_samples`. It adds no timecode conversion or audio algorithm. The output
/// path is only part of the prepared request; the application gate runs before any output is opened.
pub fn run(input_path: &Path, output_path: &Path) -> Result<(), EncodeRequestError> {
    run_with_data_rate(input_path, output_path, CodecRateProfile::Kbps1024)
}

/// Runs the selected full-file CLI pipeline at one explicitly selected supported carrier rate.
pub fn run_with_data_rate(
    input_path: &Path,
    output_path: &Path,
    codec_rate: CodecRateProfile,
) -> Result<(), EncodeRequestError> {
    run_with_data_rate_and_joc_layout(
        input_path,
        output_path,
        codec_rate,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
    )
}

/// Runs the selected full-file pipeline with one immutable JOC reconstruction layout.
pub fn run_with_data_rate_and_joc_layout(
    input_path: &Path,
    output_path: &Path,
    codec_rate: CodecRateProfile,
    joc_layout: JocReconstructionLayout,
) -> Result<(), EncodeRequestError> {
    match resolve_encode_request_route(codec_rate, joc_layout)? {
        EncodeRequestRoute::SelectedFivePointOne(profile) => run_worker(
            input_path,
            output_path,
            Some(profile.codec_rate()),
            None,
            profile.joc_reconstruction_layout(),
        ),
        EncodeRequestRoute::Flat7xI0D0Pair(_target) => Err(EncodeRequestError::Flat7xPairNotReady),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EncodeRequestRoute {
    SelectedFivePointOne(ResolvedEncodingProfile),
    Flat7xI0D0Pair(Flat7xI0D0TargetPlan),
}

fn resolve_encode_request_route(
    codec_rate: CodecRateProfile,
    joc_layout: JocReconstructionLayout,
) -> Result<EncodeRequestRoute, EncodeRequestError> {
    if codec_rate.kbps().is_none() {
        return Err(EncodeRequestError::UnsupportedDataRate);
    }
    let selected_profile = EncodingProfile::selected_with_codec_rate(codec_rate)
        .with_joc_reconstruction_layout(joc_layout);
    let resolved = resolve_selected_profile(&selected_profile)
        .map_err(|_| EncodeRequestError::UnsupportedJocLayout)?;

    match resolved.joc_reconstruction_layout() {
        JocReconstructionLayout::ExistingSelectedFivePointOne => {
            Ok(EncodeRequestRoute::SelectedFivePointOne(resolved))
        }
        JocReconstructionLayout::Flat7x => {
            let target = Flat7xI0D0TargetPlan::approved_contract();
            if target.reconstruction_layout() != JocReconstructionLayout::Flat7x
                || target.d0().rate() != resolved.codec_rate()
            {
                return Err(EncodeRequestError::UnsupportedJocLayout);
            }
            Ok(EncodeRequestRoute::Flat7xI0D0Pair(target))
        }
        JocReconstructionLayout::Unsupported => Err(EncodeRequestError::UnsupportedJocLayout),
    }
}

/// Reads an external job XML and runs the existing selected-profile CLI pipeline.
///
/// The XML remains authored input. Only its file references and the already admitted selected
/// control row are accepted; unsupported or unclosed options fail before output is opened.
pub fn run_from_job_xml(job_xml_path: &Path) -> Result<(), EncodeRequestError> {
    let request = resolve_job_xml_request(job_xml_path)?;
    run_worker(
        &request.input_path,
        &request.output_path,
        None,
        Some(request.job_xml),
        JocReconstructionLayout::ExistingSelectedFivePointOne,
    )
}

/// Validates an external job XML and its actual ADM BWF through PreparedJobInput, then stops
/// before Meter prepass, output staging, source callbacks, or encoding.
pub fn preflight_from_job_xml(job_xml_path: &Path) -> Result<(), EncodeRequestError> {
    let request = resolve_job_xml_request(job_xml_path)?;
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("jocforge-job-xml-preflight".to_owned())
            .stack_size(ENCODE_REQUEST_STACK_BYTES)
            .spawn_scoped(scope, move || run_job_xml_preflight(request))
            .map_err(|_| EncodeRequestError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| EncodeRequestError::WorkerPanicked)?
    })
}

struct ResolvedJobXmlRequest {
    job_xml_path: PathBuf,
    job_xml: Vec<u8>,
    input_path: PathBuf,
    output_path: PathBuf,
}

fn resolve_job_xml_request(
    job_xml_path: &Path,
) -> Result<ResolvedJobXmlRequest, EncodeRequestError> {
    let job_xml_path =
        std::fs::canonicalize(job_xml_path).map_err(EncodeRequestError::JobXmlInput)?;
    let job_xml = std::fs::read(&job_xml_path).map_err(EncodeRequestError::JobXmlInput)?;
    let references =
        parse_selected_job_file_references(&job_xml).map_err(EncodeRequestError::JobXmlIngress)?;
    ensure_ec3_output_container(references.output_container().is_ec3())?;
    let input_reference = references.input_reference();
    let input_path =
        resolve_job_xml_reference(input_reference.file_name(), input_reference.local_path())?;
    let output_reference = references.output_reference();
    let output_path =
        resolve_job_xml_reference(output_reference.file_name(), output_reference.local_path())?;
    if !output_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("ec3"))
    {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/job_config/output/ec3/file_name (this CLI emits elementary .ec3 output)",
        ));
    }

    Ok(ResolvedJobXmlRequest {
        job_xml_path,
        job_xml,
        input_path: normalize_existing_input(&input_path)?,
        output_path: normalize_new_output(&output_path)?,
    })
}

fn run_job_xml_preflight(request: ResolvedJobXmlRequest) -> Result<(), EncodeRequestError> {
    if request.output_path.exists() {
        return Err(EncodeRequestError::OutputAlreadyExists);
    }
    let resource = read_resource(&request.input_path).map_err(EncodeRequestError::InputFile)?;
    let parsed = parse_selected_riff_pcm24(&resource, None).map_err(EncodeRequestError::Riff)?;
    let adm = parsed
        .selected_adm_payloads()
        .map_err(EncodeRequestError::Riff)?;
    let wave = SelectedWaveDescriptor::new(parsed.sample_rate_hz(), parsed.channels(), 24);
    let programme = ProgrammeId::new("jocforge-xml-preflight")
        .map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let prepared = prepare_selected_job_input(
        programme,
        &request.job_xml,
        wave,
        parsed.total_samples(),
        Arc::clone(&resource),
        SelectedAdmResourcePayloads::new(adm.axml(), adm.chna(), adm.dbmd()),
    )
    .map_err(EncodeRequestError::PreparedInput)?;
    let data_bytes =
        u64::try_from(parsed.data_payload_bytes()).map_err(|_| EncodeRequestError::InputExtent)?;
    let prepared = prepared
        .bind_primary_control_rows(data_bytes)
        .map_err(EncodeRequestError::PreparedInput)?;
    validate_external_job_configuration(&prepared)?;

    let configuration = prepared.configuration();
    let codec_rate = configuration.selected_profile().codec_rate();
    let frame_bytes = codec_rate
        .frame_bytes()
        .ok_or(EncodeRequestError::UnsupportedDataRate)?;
    println!(
        "preflight_only=true\njob_xml_path={}\njob_xml_sha256={}\nconfig_sha256={}\ninput_kind={}\ninput_path={}\ninput_file_sha256={}\ninput_file_bytes={}\nsample_rate_hz={}\nchannels={}\npcm_bits={}\nsamples_per_channel={}\ndata_rate_kbps={}\nframe_bytes={}\noutput_path={}\noutput_exists=false\noutput_created=false\nmeter_started=false\nsource_callbacks_started=false",
        request.job_xml_path.display(),
        hex_lower(&prepared.job_xml_sha256()),
        hex_lower(&prepared.config_sha256()),
        configuration.input_kind().selected_label(),
        request.input_path.display(),
        hex_lower(&parsed.source_sha256()),
        resource.len(),
        parsed.sample_rate_hz(),
        parsed.channels(),
        24,
        parsed.total_samples(),
        configuration.bit_rate_kbps(),
        frame_bytes,
        request.output_path.display(),
    );
    Ok(())
}

fn resolve_job_xml_reference(
    file_name: &str,
    local_path: &str,
) -> Result<PathBuf, EncodeRequestError> {
    let file_name_path = Path::new(file_name);
    let mut components = file_name_path.components();
    let is_basename = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !file_name
            .chars()
            .any(|character| matches!(character, '/' | '\\' | ':'));
    let directory = Path::new(local_path);
    if file_name.is_empty() || !is_basename || !directory.is_absolute() {
        return Err(EncodeRequestError::InvalidJobXmlReference);
    }
    Ok(directory.join(file_name_path))
}

fn ensure_selected_metering_mode(value: &str) -> Result<(), EncodeRequestError> {
    if value == "1770-4" {
        Ok(())
    } else {
        Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/loudness/measure_only/metering_mode",
        ))
    }
}

fn ensure_no_temp_directory_configuration(is_configured: bool) -> Result<(), EncodeRequestError> {
    if is_configured {
        Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/job_config/misc/temp_dir",
        ))
    } else {
        Ok(())
    }
}

fn ensure_ec3_output_container(is_ec3: bool) -> Result<(), EncodeRequestError> {
    if is_ec3 {
        Ok(())
    } else {
        Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/job_config/output/mp4 or dash",
        ))
    }
}

fn validate_external_job_configuration(
    prepared: &PreparedJobInput,
) -> Result<(), EncodeRequestError> {
    let configuration = prepared.configuration();
    ensure_ec3_output_container(configuration.output_configuration().container().is_ec3())?;
    ensure_no_temp_directory_configuration(configuration.temp_directory().is_some())?;
    let loudness = configuration.loudness();
    ensure_selected_metering_mode(loudness.metering_mode())?;
    if loudness.dialogue_intelligence() || loudness.speech_threshold() != 100 {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/loudness/measure_only/dialogue_intelligence or speech_threshold",
        ));
    }

    let line_drc = configuration.line_drc_profile().xml_value();
    let rf_drc = configuration.rf_drc_profile().xml_value();
    if !((line_drc == "film_light" && rf_drc == "film_light")
        || (line_drc == "none" && rf_drc == "none"))
    {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/drc",
        ));
    }

    let downmix = configuration.downmix();
    if downmix.preferred_mode() != "loro" || downmix.mix_levels() != [-3; 4] {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/downmix",
        ));
    }
    if configuration.surround_trim_5_1().xml_value() != "auto"
        || configuration.height_trim_5_1().xml_value() != "auto"
    {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/custom_trims",
        ));
    }
    if !configuration.start().is_empty() || !configuration.end().is_empty() {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/start or end",
        ));
    }
    if configuration.input_offset() != "auto"
        || configuration.input_ffoa() != "auto"
        || configuration.time_base() != "file_position"
    {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/input/audio/{adm,atmos_mezz}/offset or ffoa, or /filter/audio/encode_to_atmos_ddp/time_base",
        ));
    }

    let prepend_is_zero = configuration
        .prepend_silence_duration()
        .parse::<f64>()
        .is_ok_and(|seconds| seconds.is_finite() && seconds == 0.0);
    let append_is_zero = configuration
        .append_silence_duration()
        .parse::<f64>()
        .is_ok_and(|seconds| seconds.is_finite() && seconds == 0.0);
    if !configuration.append_silence_duration_explicit()
        || configuration.prepend_silence_samples().value() != 0
        || configuration.append_silence_samples().value() != 0
        || !prepend_is_zero
        || !append_is_zero
    {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/prepend_silence_duration or append_silence_duration",
        ));
    }

    Ok(())
}

fn run_worker(
    input_path: &Path,
    output_path: &Path,
    codec_rate: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    joc_layout: JocReconstructionLayout,
) -> Result<(), EncodeRequestError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("jocforge-encode-request".to_owned())
            .stack_size(ENCODE_REQUEST_STACK_BYTES)
            .spawn_scoped(scope, move || {
                run_inner(
                    input_path,
                    output_path,
                    codec_rate,
                    authored_job_xml,
                    joc_layout,
                )
            })
            .map_err(|_| EncodeRequestError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| EncodeRequestError::WorkerPanicked)?
    })
}

fn run_inner(
    input_path: &Path,
    output_path: &Path,
    codec_rate_override: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    joc_layout: JocReconstructionLayout,
) -> Result<(), EncodeRequestError> {
    eprintln!("Preparing selected ADM input and owner plan...");
    let input_path = normalize_existing_input(input_path)?;
    let output_path = normalize_new_output(output_path)?;
    if output_path.exists() {
        return Err(EncodeRequestError::OutputAlreadyExists);
    }

    let is_external_job_xml = authored_job_xml.is_some();
    let generated_rate = codec_rate_override.unwrap_or(CodecRateProfile::Kbps1024);
    if generated_rate.kbps().is_none() {
        return Err(EncodeRequestError::UnsupportedDataRate);
    }
    let job_xml = match authored_job_xml {
        Some(job_xml) => job_xml,
        None => {
            let input_reference = file_reference_parts(&input_path)?;
            let output_reference = file_reference_parts(&output_path)?;
            let rate_kbps = generated_rate
                .kbps()
                .ok_or(EncodeRequestError::UnsupportedDataRate)?;
            selected_full_file_job_xml(&input_reference, &output_reference, rate_kbps).into_bytes()
        }
    };
    let run_identity = fresh_run_identity()?;
    let resource = read_resource(&input_path).map_err(EncodeRequestError::InputFile)?;
    let parsed = parse_selected_riff_pcm24(&resource, None).map_err(EncodeRequestError::Riff)?;
    let adm = parsed
        .selected_adm_payloads()
        .map_err(EncodeRequestError::Riff)?;
    let wave = SelectedWaveDescriptor::new(parsed.sample_rate_hz(), parsed.channels(), 24);
    let prepared = prepare_selected_job_input(
        run_identity.programme.clone(),
        &job_xml,
        wave,
        parsed.total_samples(),
        Arc::clone(&resource),
        SelectedAdmResourcePayloads::new(adm.axml(), adm.chna(), adm.dbmd()),
    )
    .map_err(EncodeRequestError::PreparedInput)?;
    if is_external_job_xml {
        validate_external_job_configuration(&prepared)?;
    }
    if codec_rate_override
        .is_some_and(|rate| prepared.configuration().selected_profile().codec_rate() != rate)
    {
        return Err(EncodeRequestError::UnsupportedDataRate);
    }
    let data_rate_kbps = prepared.configuration().bit_rate_kbps();
    let data_bytes =
        u64::try_from(parsed.data_payload_bytes()).map_err(|_| EncodeRequestError::InputExtent)?;
    let prepared = prepared
        .bind_primary_control_rows(data_bytes)
        .map_err(EncodeRequestError::PreparedInput)?;
    let selected_profile = prepared.configuration().selected_profile();
    let resolved_joc_profile = resolve_selected_profile(
        &EncodingProfile::selected_with_rate_and_carriage(
            selected_profile.codec_rate(),
            selected_profile.carriage(),
        )
        .with_joc_reconstruction_layout(joc_layout),
    )
    .map_err(|_| EncodeRequestError::UnsupportedJocLayout)?;
    let frame_bytes = selected_profile
        .codec_rate()
        .frame_bytes()
        .ok_or(EncodeRequestError::UnsupportedDataRate)?;
    let config_sha256 = prepared.config_sha256();

    eprintln!(
        "prepared input: attempt={} programme={} sha256={} bytes={} sample_rate_hz={} channels={} pcm_bits=24 samples_per_channel={} data_rate_kbps={} frame_bytes={} joc_reconstruction_layout={:?} joc_dmx_config_idx={:?} config_sha256={} output={}",
        run_identity.attempt.value(),
        run_identity.programme.as_str(),
        hex_lower(&parsed.source_sha256()),
        resource.len(),
        parsed.sample_rate_hz(),
        parsed.channels(),
        parsed.total_samples(),
        data_rate_kbps,
        frame_bytes,
        resolved_joc_profile.joc_reconstruction_layout(),
        resolved_joc_profile.joc_dmx_config_idx(),
        hex_lower(&config_sha256),
        output_path.display(),
    );

    let final_destination = output_path
        .to_str()
        .ok_or(EncodeRequestError::InvalidPath)?;
    let plan = jocforge_application::output_publication::admit_selected_f15_plan(
        &prepared,
        run_identity.attempt,
        &run_identity.staging_owner,
        final_destination,
    )
    .map_err(EncodeRequestError::ProductGate)?;
    let mut staging = ProgrammeOutputStaging::from_admitted_plan(&plan)
        .map_err(EncodeRequestError::OutputStaging)?;
    let output_binding = plan
        .output_binding()
        .fresh_cold_execution_binding()
        .map_err(|error| {
            EncodeRequestError::OutputStaging(OutputPublicationError::Binding(error))
        })?;

    eprintln!("Starting full-file Meter prepass...");
    let mut meter_progress = ProgressBar::new("Meter prepass");
    let meter_prepass = run_selected_meter_prepass_with_progress(
        &prepared,
        &parsed,
        run_identity.attempt,
        |completed, total| meter_progress.update(completed, total),
    )
    .map_err(EncodeRequestError::MeterPrepass)?;
    meter_progress.finish();
    let measurement = meter_prepass.dependents();
    let selected_lufs_word = measurement.receipt().selected_lufs_word();
    let selected_lufs = f32::from_bits(selected_lufs_word);
    let drc_root = cold_drc_owner_after_measurement(&prepared, measurement.receipt())
        .map_err(EncodeRequestError::DrcRoot)?;
    let source_providers = plan.source_providers();
    let metadata_owner = prepare_metadata_owner_from_measurement(
        &prepared,
        measurement,
        source_providers.metadata_owner().provider_file(),
        source_providers.metadata_owner().wrapper_file(),
    )
    .map_err(EncodeRequestError::MetadataOwner)?;
    eprintln!(
        "meter prepass complete: callbacks={} zero_drains={} selected_lufs_f32={:.6} selected_lufs_bits=0x{selected_lufs_word:08x} selected_lufs_source=ProgrammeMeasurementReceipt.selected_lufs_word selected_lufs_unit=RUST_SELECTED_LUFS_DEE_NATIVE_UNIT_UNBOUND dialnorm={:?} metadata_generation={}",
        measurement.receipt().normal_callbacks(),
        measurement.receipt().zero_drains(),
        selected_lufs,
        drc_root.state().programme_dialnorm(),
        metadata_owner.state_reference().generation(),
    );

    let supplements = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("outputs/IMPLEMENTER_SUPPLEMENTS");
    let spatial_profile = EncodingProfile::selected_with_rate_and_carriage(
        selected_profile.codec_rate(),
        selected_profile.carriage(),
    );
    let main_providers = load_selected_main_spatial_callback_providers_for_profile(
        &supplements.join("ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1"),
        &supplements.join("F01B-F03W-COLD-STATE-ASSETS-V1"),
        &supplements.join("METER-APPROVED-PARAMETER-ASSETS-V1"),
        &supplements.join("F04-RECEIVER-APPROVED-PARAMETERS-V1"),
        generate_selected_spatial_math_geometry()
            .map_err(|_| EncodeRequestError::SpatialGeometry)?,
        spatial_profile,
    )
    .map_err(EncodeRequestError::SpatialProviders)?;
    let callback_values = main_providers.callback_values().providers();
    let cold_roots = MainSpatialColdRoots::from_selected_providers(
        &prepared,
        run_identity.attempt,
        &callback_values.f01e,
        &callback_values.f03_target,
        &callback_values.p11_geometry,
    )
    .map_err(EncodeRequestError::SpatialColdRoots)?;
    let reader_eof_root = ReaderEofMachine::cold();
    let source_analysis_providers =
        SelectedSourceAnalysisProviders::load_approved().map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("F07-F10 providers: {error:?}"))
        })?;
    let analysis_root = cold_analysis_root_for_prepared_input(&prepared).map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("Analysis cold root: {error:?}"))
    })?;
    let integrity_root = cold_integrity_owner_for_plan(&prepared, &plan, run_identity.attempt)
        .map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("Integrity cold root: {error:?}"))
        })?;
    let joc_state = SelectedJocState::cold();
    let codec_root =
        CodecOwnerRoot::cold_for_rate(prepared.configuration().selected_profile().codec_rate())
            .map_err(|error| {
                EncodeRequestError::SourcePipeline(format!("Codec cold root: {error:?}"))
            })?;
    let model_dialnorm = drc_root.state().programme_dialnorm();
    let measurement_receipt = measurement.receipt();
    let roots = ProgrammeOwnerRoots::ready(
        &prepared,
        run_identity.attempt,
        measurement_receipt,
        meter_prepass.terminal_meter_root(),
        cold_roots,
        reader_eof_root,
        integrity_root,
        *analysis_root,
        drc_root,
        joc_state,
        metadata_owner,
        codec_root,
        output_binding,
    )
    .map_err(EncodeRequestError::SourcePipeline)?;

    let callback_result = (|| {
        let mut roots = roots;
        let normal_callbacks = roots.normal_callbacks().value();
        let total_callbacks = normal_callbacks.checked_add(4).ok_or_else(|| {
            EncodeRequestError::SourcePipeline("main callback count overflow".to_owned())
        })?;
        let mut committed_callbacks = 0_u64;
        let mut callback_progress = ProgressBar::new("Programme callbacks");
        callback_progress.update(0, total_callbacks);
        for callback_value in 0..total_callbacks {
            let callback_value_u32 =
                u32::try_from(callback_value).map_err(|_| EncodeRequestError::ClockMismatch)?;
            let callback_index = FrontendCallbackIndex::new(callback_value_u32);
            let pending = if callback_value < normal_callbacks {
                stage_main_callback_candidate(
                    &prepared,
                    &parsed,
                    run_identity.attempt,
                    callback_index,
                    roots.main(),
                    roots.reader_eof(),
                    &main_providers,
                )?
            } else {
                stage_main_cached_callback_candidate(
                    &prepared,
                    run_identity.attempt,
                    callback_index,
                    &roots,
                    &main_providers,
                )?
            };
            if pending.callback_index != callback_index {
                return Err(EncodeRequestError::ClockMismatch);
            }
            let callback_transaction_id = pending.transaction_id.clone();

            if let Some(source_index) = pending.source_index {
                let advance = crate::source_execution::stage_source_frame(
                    &prepared,
                    resolved_joc_profile,
                    run_identity.attempt,
                    callback_index,
                    source_index,
                    callback_transaction_id.clone(),
                    pending.frontend,
                    pending.spatial,
                    pending.reader_eof_candidate,
                    roots.main(),
                    roots.reader_eof(),
                    roots.integrity(),
                    &main_providers,
                    roots.meter(),
                    roots.measurement(),
                    roots.analysis(),
                    &source_analysis_providers,
                    roots.drc(),
                    roots.metadata(),
                    roots.joc(),
                    roots.codec(),
                    roots.output(),
                    &plan,
                    &mut staging,
                )
                .map_err(EncodeRequestError::SourcePipeline)?;
                let (next_roots, receipt) = roots
                    .commit_source_advance(callback_transaction_id, &prepared, advance)
                    .map_err(EncodeRequestError::SourcePipeline)?;
                roots = next_roots;
                committed_callbacks = committed_callbacks.checked_add(1).ok_or_else(|| {
                    EncodeRequestError::SourcePipeline("callback count overflow".to_owned())
                })?;

                if source_index.value() < 2 {
                    eprintln!(
                        "source committed: g={} callback_tx={} source={} source_tx={} disposition=suppressed frame_sha256={} owner_receipts={} phase_generation={}->{}",
                        receipt.callback_index.value(),
                        receipt.callback_transaction_id.as_str(),
                        receipt.source_index.map_or(u64::MAX, |index| index.value()),
                        receipt
                            .source_transaction_id
                            .as_ref()
                            .map_or("", |id| id.as_str()),
                        receipt
                            .frame_sha256
                            .map(|hash| hex_lower(&hash))
                            .unwrap_or_default(),
                        receipt.owner_receipts.len(),
                        receipt.phase_generation_before,
                        receipt.phase_generation_after,
                    );
                } else if let Some(append) = receipt.append.as_ref() {
                    if append.au_index == 0 {
                        eprintln!(
                            "first AU durably staged: source={} au={} frame_bytes={} frame_sha256={} emdf_container_bytes={} staging_bytes={}",
                            source_index.value(),
                            append.au_index,
                            append.bytes_written,
                            receipt
                                .frame_sha256
                                .map(|hash| hex_lower(&hash))
                                .unwrap_or_default(),
                            receipt.metadata_container_bytes.unwrap_or_default(),
                            std::fs::metadata(staging.staging_path())
                                .map_err(|error| EncodeRequestError::SourcePipeline(format!(
                                    "staged AU metadata: {error}"
                                )))?
                                .len(),
                        );
                    }
                    if append.au_index == 0
                        || append.au_index % 512 == 0
                        || append.au_index + 1 == roots.expected_published()
                    {
                        eprintln!(
                            "AU append committed: g={} callback_tx={} source={} source_tx={} au={} frame_bytes={} crc_residual={} durable={} published={}/{} phase_generation={}->{}",
                            receipt.callback_index.value(),
                            receipt.callback_transaction_id.as_str(),
                            receipt.source_index.map_or(u64::MAX, |index| index.value()),
                            receipt
                                .source_transaction_id
                                .as_ref()
                                .map_or("", |id| id.as_str()),
                            receipt.au_index.unwrap_or(u64::MAX),
                            append.bytes_written,
                            append.crc_residual,
                            append.durable,
                            roots.published_aus(),
                            roots.expected_published(),
                            receipt.phase_generation_before,
                            receipt.phase_generation_after,
                        );
                    }
                } else {
                    return Err(EncodeRequestError::SourcePipeline(
                        "published source committed without a durable append receipt".to_owned(),
                    ));
                }
            } else {
                let frontend = pending.frontend.ok_or(EncodeRequestError::ClockMismatch)?;
                let spatial_parent = roots
                    .main()
                    .prepare_spatial_after_callback(
                        &prepared,
                        frontend,
                        pending.spatial,
                        callback_transaction_id.clone(),
                    )
                    .map_err(|error| {
                        EncodeRequestError::SourcePipeline(format!(
                            "callback-only F01E/F03W/F04 proposal: {error:?}"
                        ))
                    })?;
                let (main_after, parent_transaction_id, parent_callback, spatial_receipts) =
                    spatial_parent.into_parts();
                if parent_transaction_id != callback_transaction_id
                    || parent_callback != callback_index
                {
                    return Err(EncodeRequestError::ClockMismatch);
                }
                let (reader_after, _, reader_receipt) = pending.reader_eof_candidate.into_parts();
                let changed_receipts = [
                    spatial_receipts[0].clone(),
                    spatial_receipts[1].clone(),
                    spatial_receipts[2].clone(),
                    reader_receipt,
                ];
                let (next_roots, receipt) = roots
                    .commit_callback_only(
                        &prepared,
                        callback_index,
                        callback_transaction_id,
                        main_after,
                        reader_after,
                        changed_receipts,
                    )
                    .map_err(EncodeRequestError::SourcePipeline)?;
                roots = next_roots;
                committed_callbacks = committed_callbacks.checked_add(1).ok_or_else(|| {
                    EncodeRequestError::SourcePipeline("callback count overflow".to_owned())
                })?;
                eprintln!(
                    "priming callback committed: g={} callback_tx={} owner_receipts={} phase_generation={}->{}",
                    receipt.callback_index.value(),
                    receipt.callback_transaction_id.as_str(),
                    receipt.owner_receipts.len(),
                    receipt.phase_generation_before,
                    receipt.phase_generation_after,
                );
            }
            callback_progress.update(callback_value + 1, total_callbacks);
        }
        callback_progress.finish();

        if committed_callbacks != total_callbacks || !roots.is_complete_for_eof() {
            return Err(EncodeRequestError::SourcePipeline(
                "callback/source/AU clocks did not reach the selected EOF boundary".to_owned(),
            ));
        }
        Ok((roots, committed_callbacks))
    })();
    let (mut roots, committed_callbacks) = match callback_result {
        Ok(value) => value,
        Err(error) => {
            staging.abandon_attempt();
            return Err(error);
        }
    };
    if !roots.is_complete_for_eof() {
        staging.abandon_attempt();
        return Err(EncodeRequestError::SourcePipeline(
            "callback/source/AU clocks did not reach the selected EOF boundary".to_owned(),
        ));
    }
    let eof_transaction_id = CallbackTransactionId::new(&format!(
        "jocforge-eof-attempt-{}",
        run_identity.attempt.value()
    ))
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("EOF transaction identity: {error:?}"))
    })?;
    let eof_proposal = match roots.prepare_eof_proposal() {
        Ok(proposal) => proposal,
        Err(error) => {
            staging.abandon_attempt();
            return Err(EncodeRequestError::SourcePipeline(error));
        }
    };
    let (finalizing_roots, metadata_eof_receipt, finalize_proposal) =
        match roots.begin_finalizing(eof_proposal, eof_transaction_id) {
            Ok(value) => value,
            Err(error) => {
                staging.abandon_attempt();
                return Err(EncodeRequestError::SourcePipeline(error));
            }
        };
    roots = finalizing_roots;
    eprintln!(
        "EOF committed: sources={} published={} bytes={} pending_drop_receipts={} metadata_generation={} phase_generation={}",
        roots.source_index().value(),
        roots.published_aus(),
        roots.durable_bytes(),
        7,
        roots.metadata().state_reference().generation(),
        roots.phase_generation(),
    );
    let _metadata_eof_receipt = metadata_eof_receipt;

    eprintln!("Finalizing and exposing output...");
    let finalized =
        finalize_output(&mut staging, roots.output(), finalize_proposal).map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("F15 finalize/exposure failed: {error}"))
        })?;
    let (done_roots, exposure_receipt) = roots
        .finish(finalized)
        .map_err(EncodeRequestError::SourcePipeline)?;
    roots = done_roots;
    if !roots.is_done() || exposure_receipt.product_length() != roots.expected_product_bytes() {
        return Err(EncodeRequestError::SourcePipeline(
            "DONE root does not match the durable final exposure receipt".to_owned(),
        ));
    }
    let final_length = std::fs::metadata(&output_path)
        .map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("final output metadata: {error}"))
        })?
        .len();
    if final_length != exposure_receipt.product_length() {
        return Err(EncodeRequestError::SourcePipeline(
            "final output length differs from FinalExposureReceipt".to_owned(),
        ));
    }
    eprintln!(
        "programme complete: status=DONE output={} bytes={} sha256={} callbacks={}/{} sources={}/{} published_aus={} phase_generation={} model_dialnorm={:?} native_dialnorm_parity=UNBOUND render=NOT_RUN playback=NOT_RUN",
        output_path.display(),
        exposure_receipt.product_length(),
        hex_lower(exposure_receipt.product_sha256()),
        committed_callbacks,
        u64::from(roots.callback_index().value()),
        roots.source_index().value(),
        roots.expected_sources(),
        roots.published_aus(),
        roots.phase_generation(),
        model_dialnorm,
    );
    Ok(())
}

struct PendingMainCallback {
    callback_index: FrontendCallbackIndex,
    source_index: Option<CodecSourceIndex>,
    frontend: Option<FrontendControlReplacementCandidate>,
    spatial: MainSpatialCallbackCandidate,
    reader_eof_candidate: ReaderEofReplacementCandidate,
    transaction_id: CallbackTransactionId,
}

fn stage_main_callback_candidate(
    prepared: &PreparedJobInput,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    attempt: AttemptId,
    callback_index: FrontendCallbackIndex,
    roots: &MainSpatialColdRoots,
    reader_eof_before: &ReaderEofMachine,
    providers: &crate::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
) -> Result<PendingMainCallback, EncodeRequestError> {
    let frame = read_frontend_pcm_callback(
        parsed,
        prepared.resource_identity().input_sha256(),
        callback_index,
    )
    .map_err(EncodeRequestError::FrontendPcm)?;
    let transaction_id = CallbackTransactionId::new(&format!(
        "jocforge-main-callback-g{}",
        callback_index.value()
    ))
    .map_err(|_| EncodeRequestError::CallbackTransaction)?;
    let candidate = stage_main_frontend_adm_input_snapshot(
        prepared,
        roots.frontend_control(),
        roots.normal_control_prepared(),
        attempt,
        transaction_id.clone(),
    )
    .map_err(EncodeRequestError::FrontendSnapshot)?;
    let (frontend, controls) = candidate.into_parts();
    let actual_samples = ReaderEofActualSampleCount::new(
        u32::try_from(frame.actual_samples().value())
            .map_err(|_| EncodeRequestError::ClockMismatch)?,
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof actual sample count: {error:?}"))
    })?;
    let padding_samples = ReaderEofPaddingSampleCount::new(
        u32::try_from(frame.padding_samples()).map_err(|_| EncodeRequestError::ClockMismatch)?,
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof padding sample count: {error:?}"))
    })?;
    let reader_eof_clock = ReaderEofNormalCallbackClock::new(
        frame.callback_index(),
        frame.first_sample(),
        actual_samples,
        padding_samples,
        controls.normal_callbacks(),
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof callback clock: {error:?}"))
    })?;
    let reader_eof_candidate = stage_reader_eof_adm_input_normal_callback(
        reader_eof_before,
        reader_eof_clock,
        frame.physical_pcm_rows(),
        &controls,
        transaction_id.clone(),
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof callback stage: {error:?}"))
    })?;
    let values = providers.callback_values().providers();
    let spatial = stage_main_spatial_callback(MainSpatialCallbackInput {
        prepared,
        controls: MainNormalControlOutput::AdmInput(controls),
        physical_pcm_rows: frame.physical_pcm_rows(),
        spatial_control: roots.spatial_control(),
        spatial_audio: roots.spatial_audio(),
        p11_geometry: &values.p11_geometry,
        p15_geometry: &values.p15_geometry,
        diffusion_parameters: &values.diffusion,
        feature_processor: &values.feature_processor,
        feature_fft: &values.feature_fft,
        window: &values.window,
        component_parameters: &values.component_parameters,
        spatial_audio_providers: Some(providers.spatial_audio()),
    })
    .map_err(EncodeRequestError::SpatialCallback)?;
    if spatial.callback_index() != callback_index {
        return Err(EncodeRequestError::ClockMismatch);
    }
    Ok(PendingMainCallback {
        callback_index,
        source_index: frame.source_index(),
        frontend: Some(frontend),
        spatial,
        reader_eof_candidate,
        transaction_id,
    })
}

fn stage_main_cached_callback_candidate(
    prepared: &PreparedJobInput,
    attempt: AttemptId,
    callback_index: FrontendCallbackIndex,
    roots: &ProgrammeOwnerRoots<'_>,
    providers: &crate::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
) -> Result<PendingMainCallback, EncodeRequestError> {
    let source_index =
        source_for_callback_clock(callback_index).ok_or(EncodeRequestError::ClockMismatch)?;
    let transaction_id = CallbackTransactionId::new(&format!(
        "jocforge-main-callback-g{}",
        callback_index.value()
    ))
    .map_err(|_| EncodeRequestError::CallbackTransaction)?;
    let callback_lineage =
        CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
    let frontend_control = roots.main().frontend_control();
    let after = frontend_control
        .committed()
        .dynamic_after_state()
        .ok_or_else(|| {
            EncodeRequestError::SourcePipeline("MAIN F01E after-state unavailable".to_owned())
        })?;
    let held_controls = MainHeldControlOutput::AdmInput(
        after
            .held_source_view_for_cached_callback(callback_lineage)
            .map_err(|error| {
                EncodeRequestError::SourcePipeline(format!(
                    "MAIN cached F01E held controls: {error:?}"
                ))
            })?,
    );
    let reader_eof_candidate = stage_reader_eof_cached_drain(
        roots.reader_eof(),
        callback_index,
        roots.normal_callbacks(),
        transaction_id.clone(),
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof cached drain: {error:?}"))
    })?;

    let values = providers.callback_values().providers();
    let spatial = stage_main_spatial_cached_callback(MainSpatialCachedCallbackInput {
        prepared,
        controls: held_controls,
        spatial_control: roots.main().spatial_control(),
        spatial_audio: roots.main().spatial_audio(),
        p11_geometry: &values.p11_geometry,
        p15_geometry: &values.p15_geometry,
        diffusion_parameters: &values.diffusion,
        feature_processor: &values.feature_processor,
        feature_fft: &values.feature_fft,
        window: &values.window,
        component_parameters: &values.component_parameters,
        spatial_audio_providers: Some(providers.spatial_audio()),
    })
    .map_err(EncodeRequestError::SpatialCallback)?;
    if spatial.callback_index() != callback_index {
        return Err(EncodeRequestError::ClockMismatch);
    }
    Ok(PendingMainCallback {
        callback_index,
        source_index: Some(source_index),
        frontend: None,
        spatial,
        reader_eof_candidate,
        transaction_id,
    })
}

struct FileReferenceParts {
    file_name: String,
    local_path: String,
}

fn normalize_existing_input(path: &Path) -> Result<PathBuf, EncodeRequestError> {
    if !path.is_absolute() {
        return Err(EncodeRequestError::InvalidPath);
    }
    std::fs::canonicalize(path).map_err(|_| EncodeRequestError::InvalidPath)
}

fn normalize_new_output(path: &Path) -> Result<PathBuf, EncodeRequestError> {
    if !path.is_absolute() {
        return Err(EncodeRequestError::InvalidPath);
    }
    let file_name = path.file_name().ok_or(EncodeRequestError::InvalidPath)?;
    let parent = path.parent().ok_or(EncodeRequestError::InvalidPath)?;
    let normalized_parent =
        std::fs::canonicalize(parent).map_err(|_| EncodeRequestError::InvalidPath)?;
    Ok(normalized_parent.join(file_name))
}

fn file_reference_parts(path: &Path) -> Result<FileReferenceParts, EncodeRequestError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(EncodeRequestError::InvalidPath)?;
    let local_path = path
        .parent()
        .and_then(Path::to_str)
        .filter(|value| !value.is_empty())
        .ok_or(EncodeRequestError::InvalidPath)?;
    Ok(FileReferenceParts {
        file_name: xml_escape(file_name),
        local_path: xml_escape(local_path),
    })
}

fn selected_full_file_job_xml(
    input: &FileReferenceParts,
    output: &FileReferenceParts,
    data_rate_kbps: u32,
) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><job_config><input><audio><adm><file_name>{}</file_name><storage><local><path>{}</path></local></storage><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa></adm></audio></input><filter><audio><encode_to_atmos_ddp><loudness><measure_only><metering_mode>1770-4</metering_mode><dialogue_intelligence>false</dialogue_intelligence><speech_threshold>100</speech_threshold></measure_only></loudness><data_rate>{}</data_rate><timecode_frame_rate>not_indicated</timecode_frame_rate><start></start><end></end><time_base>file_position</time_base><prepend_silence_duration>0.0</prepend_silence_duration><append_silence_duration>0.0</append_silence_duration><drc><line_mode_drc_profile>film_light</line_mode_drc_profile><rf_mode_drc_profile>film_light</rf_mode_drc_profile></drc><downmix><loro_center_mix_level>-3</loro_center_mix_level><loro_surround_mix_level>-3</loro_surround_mix_level><ltrt_center_mix_level>-3</ltrt_center_mix_level><ltrt_surround_mix_level>-3</ltrt_surround_mix_level><preferred_downmix_mode>loro</preferred_downmix_mode></downmix><custom_trims><surround_trim_5_1>auto</surround_trim_5_1><height_trim_5_1>auto</height_trim_5_1></custom_trims></encode_to_atmos_ddp></audio></filter><output><ec3><file_name>{}</file_name><storage><local><path>{}</path></local></storage></ec3></output></job_config>",
        input.file_name, input.local_path, data_rate_kbps, output.file_name, output.local_path
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

struct CliRunIdentity {
    programme: ProgrammeId,
    attempt: AttemptId,
    staging_owner: OutputStagingOwnerId,
}

fn fresh_run_identity() -> Result<CliRunIdentity, EncodeRequestError> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EncodeRequestError::ProgrammeIdentity)?
        .as_nanos();
    let attempt = u64::try_from(nonce).map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let unique_label = format!("jocforge-cli-{}-{nonce}", std::process::id());
    let programme_id =
        ProgrammeId::new(&unique_label).map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let staging_owner_id = OutputStagingOwnerId::new(&format!(
        "jocforge-output-owner-{}-{nonce}",
        std::process::id()
    ))
    .map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    Ok(CliRunIdentity {
        programme: programme_id,
        attempt: AttemptId::new(attempt),
        staging_owner: staging_owner_id,
    })
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        EncodeRequestError, EncodeRequestRoute, FileReferenceParts, ensure_ec3_output_container,
        ensure_no_temp_directory_configuration, ensure_selected_metering_mode,
        resolve_encode_request_route, run_with_data_rate, run_with_data_rate_and_joc_layout,
        selected_full_file_job_xml,
    };
    use jocforge_application::meter_source_bridge::CodecRateProfile;
    use jocforge_application::meter_source_bridge::JocReconstructionLayout;
    use std::path::Path;

    #[test]
    fn generated_job_xml_preserves_each_explicit_carrier_rate() {
        let input = FileReferenceParts {
            file_name: "input.wav".to_owned(),
            local_path: "D:\\audio".to_owned(),
        };
        let output = FileReferenceParts {
            file_name: "output.ec3".to_owned(),
            local_path: "D:\\audio".to_owned(),
        };
        let rows = [
            (CodecRateProfile::Kbps384, 384),
            (CodecRateProfile::Kbps448, 448),
            (CodecRateProfile::Kbps576, 576),
            (CodecRateProfile::Kbps640, 640),
            (CodecRateProfile::Kbps768, 768),
            (CodecRateProfile::Kbps1024, 1_024),
        ];

        for (codec_rate, data_rate_kbps) in rows {
            let xml = selected_full_file_job_xml(&input, &output, data_rate_kbps);
            assert!(xml.contains(&format!("<data_rate>{data_rate_kbps}</data_rate>")));
            assert_eq!(
                xml.matches("<data_rate>").count(),
                1,
                "the CLI emits a single rate token"
            );
            assert_eq!(codec_rate.kbps(), Some(data_rate_kbps));
        }
    }

    #[test]
    fn unsupported_rate_rejects_before_input_or_output_io() {
        assert!(matches!(
            run_with_data_rate(
                Path::new("missing-input.wav"),
                Path::new("unused-output.ec3"),
                CodecRateProfile::Unsupported,
            ),
            Err(EncodeRequestError::UnsupportedDataRate)
        ));
    }

    #[test]
    fn flat7x_rate_constraint_rejects_before_input_or_output_io() {
        assert!(matches!(
            run_with_data_rate_and_joc_layout(
                Path::new("missing-input.wav"),
                Path::new("unused-output.ec3"),
                CodecRateProfile::Kbps448,
                JocReconstructionLayout::Flat7x,
            ),
            Err(EncodeRequestError::UnsupportedJocLayout)
        ));
    }

    #[test]
    fn flat7x_selects_the_approved_i0_d0_pair_target() {
        let route = resolve_encode_request_route(
            CodecRateProfile::Kbps1024,
            JocReconstructionLayout::Flat7x,
        )
        .unwrap();
        let EncodeRequestRoute::Flat7xI0D0Pair(target) = route else {
            panic!("Flat7x must select the paired target route");
        };

        assert_eq!(target.i0().rate(), CodecRateProfile::Kbps640);
        assert_eq!(target.i0().frame_bytes(), 2_560);
        assert_eq!(target.d0().rate(), CodecRateProfile::Kbps1024);
        assert_eq!(target.d0().frame_bytes(), 4_096);
        assert_eq!(target.aggregate_rate_kbps(), Some(1_664));
        assert_eq!(target.aggregate_frame_bytes(), 6_656);
        assert!(!target.production_allowed());
    }

    #[test]
    fn selected_five_point_one_keeps_every_supported_rate_on_its_single_frame_route() {
        let rows = [
            CodecRateProfile::Kbps384,
            CodecRateProfile::Kbps448,
            CodecRateProfile::Kbps576,
            CodecRateProfile::Kbps640,
            CodecRateProfile::Kbps768,
            CodecRateProfile::Kbps1024,
        ];

        for codec_rate in rows {
            let route = resolve_encode_request_route(
                codec_rate,
                JocReconstructionLayout::ExistingSelectedFivePointOne,
            )
            .unwrap();
            let EncodeRequestRoute::SelectedFivePointOne(profile) = route else {
                panic!("the selected 5.1 profile must retain the single-frame route");
            };
            assert_eq!(profile.codec_rate(), codec_rate);
            assert_eq!(
                profile.joc_reconstruction_layout(),
                JocReconstructionLayout::ExistingSelectedFivePointOne
            );
        }

        assert!(matches!(
            run_with_data_rate(
                Path::new("missing-input.wav"),
                Path::new("unused-output.ec3"),
                CodecRateProfile::Kbps1024,
            ),
            Err(EncodeRequestError::InvalidPath)
        ));
    }

    #[test]
    fn flat7x_fails_closed_before_input_or_output_staging() {
        assert!(matches!(
            run_with_data_rate_and_joc_layout(
                Path::new("missing-input.wav"),
                Path::new("unused-output.ec3"),
                CodecRateProfile::Kbps1024,
                JocReconstructionLayout::Flat7x,
            ),
            Err(EncodeRequestError::Flat7xPairNotReady)
        ));
    }

    #[test]
    fn selected_cli_gate_rejects_nondefault_metering_modes() {
        assert!(ensure_selected_metering_mode("1770-4").is_ok());
        for mode in ["1770-3", "1770-2", "1770-1", "LeqA"] {
            assert!(matches!(
                ensure_selected_metering_mode(mode),
                Err(EncodeRequestError::UnsupportedJobXmlOption(
                    "/filter/audio/encode_to_atmos_ddp/loudness/measure_only/metering_mode"
                ))
            ));
        }
    }

    #[test]
    fn selected_cli_gate_rejects_authored_temp_directory() {
        assert!(ensure_no_temp_directory_configuration(false).is_ok());
        assert!(matches!(
            ensure_no_temp_directory_configuration(true),
            Err(EncodeRequestError::UnsupportedJobXmlOption(
                "/job_config/misc/temp_dir"
            ))
        ));
    }

    #[test]
    fn selected_cli_gate_accepts_only_elementary_ec3_output() {
        assert!(ensure_ec3_output_container(true).is_ok());
        assert!(matches!(
            ensure_ec3_output_container(false),
            Err(EncodeRequestError::UnsupportedJobXmlOption(
                "/job_config/output/mp4 or dash"
            ))
        ));
    }
}
