// pattern: Imperative Shell
//! Reads one selected ADM BWF, prepares its input identity, and enters the application encode gate.

#[cfg(not(target_arch = "wasm32"))]
pub use crate::cmaf::{write_composed_m4a, write_m4a_with_object_count};
pub use jocforge_application::source_frame_bridge::{
    BlockPartition, EmdfLocation, LfeOwner, SubstreamPlan,
};
pub use jocforge_application::{FileRenderTiming, ResourceIdentity, SourceTimeRangeError};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
    thread,
};

use crate::meter_prepass::{SelectedMeterPrepassError, unpack_provider_supplements};
use crate::output_publication::{
    OutputPublicationError, ProgrammeOutputStaging, capture_flat7x_experimental_candidate,
    finalize_output,
};
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
        CarrierCodecProfile, CodecRateProfile, EmdfCarriageProfile, EncodingProfile,
        Flat7xFrameCodec, Flat7xI0D0TargetPlan, Flat7xSubstreamRole,
        FrontendControlReplacementCandidate, JocReconstructionLayout, ResolvedEncodingProfile,
        SubstreamTopology, resolve_selected_profile,
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
    source_frame_bridge::{CodecOwnerRoot, Flat7xD0PcmSourceExtent},
    spatial_callback::{
        MainFrontendSnapshotError, MainHeldControlOutput, MainNormalControlOutput,
        MainSpatialCachedCallbackInput, MainSpatialCallbackCandidate, MainSpatialCallbackError,
        MainSpatialCallbackInput, MainSpatialColdRoots, MainSpatialColdRootsError,
        MainSpatialFrame, stage_main_frontend_adm_input_snapshot,
        stage_main_spatial_cached_callback_on_current_thread,
        stage_main_spatial_callback_on_current_thread,
    },
};
use jocforge_io::pcm::{
    frontend_pcm_source::{
        CodecSourceIndex, FrontendCallbackIndex, FrontendPcmSourceError, read_frontend_pcm_callback,
    },
    riff_pcm24::{parse_selected_riff_pcm24, parse_selected_riff_pcm24_streamed},
    riff_pcm24_file::{RiffPcm24FileReadError, read_resource},
};

const ENCODE_REQUEST_STACK_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub enum EncodeRequestError {
    Cancelled,
    InvalidPath,
    JobXmlInput(std::io::Error),
    JobXmlIngress(JobXmlIngressError),
    InvalidJobXmlReference,
    UnsupportedJobXmlOption(&'static str),
    OutputAlreadyExists,
    UnsupportedDataRate,
    UnsupportedJocLayout,
    ExperimentalCandidatePathRequired,
    ExperimentalCandidateDirectoryNotEmpty,
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
            Self::Cancelled => formatter.write_str("encoding cancelled"),
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
                formatter.write_str("joc-layout flat7x requires a supported D0 data rate")
            }
            Self::ExperimentalCandidatePathRequired => formatter.write_str(
                "Flat7x output must use a .candidate.ec3 filename in a unique attempt directory",
            ),
            Self::ExperimentalCandidateDirectoryNotEmpty => formatter
                .write_str("Flat7x candidate output requires a new empty attempt directory"),
            Self::InputFile(error) => write!(formatter, "{error}"),
            Self::Riff(error) => write!(formatter, "WAVE input rejected: {error}"),
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

fn map_source_frame_stage_error(
    error: crate::source_execution::SourceFrameStageError,
) -> EncodeRequestError {
    match error {
        crate::source_execution::SourceFrameStageError::Pipeline(message) => {
            EncodeRequestError::SourcePipeline(message)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ComposedOutput = tempfile::NamedTempFile;
#[cfg(target_arch = "wasm32")]
pub(crate) type ComposedOutput = std::io::Cursor<Vec<u8>>;

/// What one finished request published for its caller.
///
/// The file targets expose the product through the namespace and report only success. A target
/// without a namespace hands the assembled bytes back instead of pretending a path exists.
#[derive(Debug)]
pub(crate) enum RunProduct {
    Published,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Bytes(Vec<u8>),
}

/// Encodes one complete ADM BWF from memory and returns the elementary stream.
///
/// Native execution stages the resource in an attempt directory and reuses the file pipeline. The
/// browser target has no directory: it feeds the caller's bytes straight into the same request and
/// returns the product the in-isolate publication owner assembled.
pub fn encode_adm_bytes(adm: &[u8]) -> Result<Vec<u8>, EncodeRequestError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let directory = tempfile::tempdir()
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()))?;
        let input = directory.path().join("input.wav");
        let output = directory.path().join("output.ec3");
        std::fs::write(&input, adm)
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()))?;
        run(&input, &output)?;
        return std::fs::read(&output)
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()));
    }
    #[cfg(target_arch = "wasm32")]
    {
        encode_adm_resource(std::sync::Arc::from(adm), 1024, 15, None, 0, None, 1)
    }
}

/// Encodes one ADM BWF the host streams with no composed topology.
#[cfg(target_arch = "wasm32")]
pub fn encode_adm_host_streamed(
    rate_kbps: u32,
    object_count: u8,
    job_xml: Option<Vec<u8>>,
    layout: u8,
    surround_ex: u8,
) -> Result<Vec<u8>, EncodeRequestError> {
    encode_browser_request(
        None,
        rate_kbps,
        object_count,
        job_xml,
        layout,
        None,
        surround_ex,
    )
}

/// Encodes a complete ADM BWF whose bytes the caller already owns.
#[cfg(target_arch = "wasm32")]
pub fn encode_adm_resource(
    resource: std::sync::Arc<[u8]>,
    rate_kbps: u32,
    object_count: u8,
    job_xml: Option<Vec<u8>>,
    layout: u8,
    dependent_count: Option<u8>,
    surround_ex: u8,
) -> Result<Vec<u8>, EncodeRequestError> {
    encode_browser_request(
        Some(resource),
        rate_kbps,
        object_count,
        job_xml,
        layout,
        dependent_count,
        surround_ex,
    )
}

/// Encodes from resident bytes or the host reader using the same requested signalling.
#[cfg(target_arch = "wasm32")]
pub fn encode_browser_request(
    resource: Option<Arc<[u8]>>,
    rate_kbps: u32,
    object_count: u8,
    job_xml: Option<Vec<u8>>,
    layout: u8,
    dependent_count: Option<u8>,
    surround_ex: u8,
) -> Result<Vec<u8>, EncodeRequestError> {
    let layout = match layout {
        0 => JocReconstructionLayout::ExistingSelectedFivePointOne,
        1 => JocReconstructionLayout::Flat7x,
        _ => return Err(EncodeRequestError::UnsupportedJocLayout),
    };
    let rate =
        CodecRateProfile::from_kbps(rate_kbps).ok_or(EncodeRequestError::UnsupportedDataRate)?;
    let objects = match object_count {
        8 => JocObjectCount::Eight,
        12 => JocObjectCount::Twelve,
        15 => JocObjectCount::Fifteen,
        _ => return Err(EncodeRequestError::UnsupportedDataRate),
    };
    // A dependent count is the only composition axis the browser needs: the plan itself is built
    // here from the selected layout, so no topology type crosses the crate boundary.
    let topology = match dependent_count {
        Some(count) => Some(
            jocforge_application::source_frame_bridge::SubstreamPlan::new(
                layout,
                count,
                LfeOwner::Independent,
            )
            .and_then(|plan| plan.with_emdf_location(EmdfLocation::SkipField))
            .map(|plan| plan.with_partition(BlockPartition::Six))
            .map_err(|error| {
                EncodeRequestError::SourcePipeline(format!("browser topology rejected: {error}"))
            })?,
        ),
        None => None,
    };
    let product = run_worker_with_object_count(
        Path::new("/jocforge-browser/input.wav"),
        Path::new("memory://output.ec3"),
        Some(rate),
        job_xml,
        (layout, EmdfCarriageProfile::DeployedCompat),
        None,
        &std::sync::atomic::AtomicBool::new(false),
        topology,
        objects,
        surround_ex,
        resource,
    )?;
    match product {
        RunProduct::Bytes(bytes) => Ok(bytes),
        RunProduct::Published => Err(EncodeRequestError::SourcePipeline(
            "browser request produced no in-memory product".to_owned(),
        )),
    }
}

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
    run_with_output_signalling(
        input_path,
        output_path,
        codec_rate,
        joc_layout,
        EmdfCarriageProfile::DeployedCompat,
    )
}

pub fn run_with_output_signalling(
    input_path: &Path,
    output_path: &Path,
    codec_rate: CodecRateProfile,
    joc_layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
) -> Result<(), EncodeRequestError> {
    match resolve_encode_request_route(codec_rate, joc_layout)? {
        EncodeRequestRoute::SelectedFivePointOne(profile) => run_worker(
            input_path,
            output_path,
            Some(profile.codec_rate()),
            None,
            (profile.joc_reconstruction_layout(), signalling),
        ),
        EncodeRequestRoute::Flat7xI0D0Pair(target) => run_worker(
            input_path,
            output_path,
            Some(target.d0().rate()),
            None,
            (target.reconstruction_layout(), signalling),
        ),
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
    let selected_profile = EncodingProfile::selected_with_codec_rate_and_joc_reconstruction_layout(
        codec_rate, joc_layout,
    );
    let resolved = resolve_selected_profile(&selected_profile)
        .map_err(|_| EncodeRequestError::UnsupportedJocLayout)?;

    match resolved.carrier_codec() {
        CarrierCodecProfile::IndependentEac3 => {
            Ok(EncodeRequestRoute::SelectedFivePointOne(resolved))
        }
        CarrierCodecProfile::Flat7xAc3I0DependentEac3D0 => {
            let target = Flat7xI0D0TargetPlan::for_layout(codec_rate, joc_layout)
                .ok_or(EncodeRequestError::UnsupportedJocLayout)?;
            if !resolved.joc_reconstruction_layout().uses_seven_carriers()
                || resolved.substream_topology()
                    != SubstreamTopology::Flat7xIndependentI0DependentD0
                || resolved.joc_dmx_config_idx() != Some(target.joc_dmx_config_idx())
                || !resolved.joc_payload_downmix_index().uses_seven_carriers()
                || target.reconstruction_layout() != joc_layout
                || target.i0().role() != Flat7xSubstreamRole::IndependentI0
                || target.i0().codec() != Flat7xFrameCodec::Ac3
                || target.i0().bsid() != 6
                || target.d0().role() != Flat7xSubstreamRole::DependentD0
                || target.d0().codec() != Flat7xFrameCodec::Eac3
                || target.d0().bsid() != 16
                || target.d0().rate() != resolved.codec_rate()
            {
                return Err(EncodeRequestError::UnsupportedJocLayout);
            }
            Ok(EncodeRequestRoute::Flat7xI0D0Pair(target))
        }
        CarrierCodecProfile::Unsupported => Err(EncodeRequestError::UnsupportedJocLayout),
    }
}

/// Reads an external job XML and runs the existing selected-profile CLI pipeline.
///
/// The XML remains authored input. Only its file references and the already admitted selected
/// control row are accepted; unsupported or unclosed options fail before output is opened.
pub fn run_from_job_xml(job_xml_path: &Path) -> Result<(), EncodeRequestError> {
    encode_job_xml(
        job_xml_path,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        false,
    )
}

/// Validates an external job XML and its actual ADM BWF through PreparedJobInput, then stops
/// before Meter prepass, output staging, source callbacks, or encoding.
pub fn preflight_from_job_xml(job_xml_path: &Path) -> Result<(), EncodeRequestError> {
    encode_job_xml(
        job_xml_path,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        true,
    )
}

/// Runs an XML job for the chosen output layout, or validates its timeline and output location.
pub fn encode_job_xml(
    job_xml_path: &Path,
    layout: JocReconstructionLayout,
    preflight_only: bool,
) -> Result<(), EncodeRequestError> {
    encode_job_xml_with_signalling(
        job_xml_path,
        layout,
        preflight_only,
        EmdfCarriageProfile::DeployedCompat,
    )
}

pub fn encode_job_xml_with_signalling(
    job_xml_path: &Path,
    layout: JocReconstructionLayout,
    preflight_only: bool,
    signalling: EmdfCarriageProfile,
) -> Result<(), EncodeRequestError> {
    encode_xml_request(
        job_xml_path,
        layout,
        preflight_only,
        signalling,
        false,
        None,
    )
}
fn encode_xml_request(
    job_xml_path: &Path,
    layout: JocReconstructionLayout,
    preflight_only: bool,
    signalling: EmdfCarriageProfile,
    ac3_core: bool,
    cmaf_rate: Option<u32>,
) -> Result<(), EncodeRequestError> {
    validate_signalling(layout, signalling)?;
    let request = resolve_job_xml_request(job_xml_path, cmaf_rate.is_some())?;
    if !preflight_only {
        return run_worker(
            &request.input_path,
            &request.output_path,
            None,
            Some(request.job_xml),
            (layout, signalling),
        );
    }
    if cmaf_rate.is_none() && layout.uses_seven_carriers() {
        validate_flat7x_experimental_candidate_path(&request.output_path)?;
    }
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name("jocforge-job-xml-preflight".to_owned())
            .stack_size(ENCODE_REQUEST_STACK_BYTES)
            .spawn_scoped(scope, move || {
                run_job_xml_preflight(&request, layout, ac3_core, cmaf_rate)
            })
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
    cmaf: bool,
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
    if !cmaf
        && !output_path
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

fn run_job_xml_preflight(
    request: &ResolvedJobXmlRequest,
    layout: JocReconstructionLayout,
    ac3_core: bool,
    cmaf_rate: Option<u32>,
) -> Result<(), EncodeRequestError> {
    if request.output_path.exists() {
        return Err(EncodeRequestError::OutputAlreadyExists);
    }
    let resource = read_resource(&request.input_path).map_err(EncodeRequestError::InputFile)?;
    let parsed = parse_selected_riff_pcm24(&resource, None).map_err(EncodeRequestError::Riff)?;
    let adm = parsed
        .selected_adm_payloads()
        .map_err(EncodeRequestError::Riff)?;
    let wave = SelectedWaveDescriptor::new(
        parsed.sample_rate_hz(),
        parsed.channels(),
        parsed.pcm_bits(),
    );
    let programme = ProgrammeId::new("jocforge-xml-preflight")
        .map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let prepared = prepare_selected_job_input(
        programme,
        &request.job_xml,
        wave,
        parsed.total_samples(),
        ResourceIdentity::new(parsed.source_sha256(), parsed.resource_bytes()),
        SelectedAdmResourcePayloads::new(
            adm.axml(),
            adm.axml_offset(),
            adm.chna(),
            adm.chna_offset(),
            adm.dbmd(),
            adm.dbmd_offset(),
        ),
    )
    .map_err(EncodeRequestError::PreparedInput)?;
    let parsed = parsed
        .with_adm_channel_projection(
            prepared.graph().graph().bed_mix_sources(),
            prepared.graph().graph().unused_file_channels(),
        )
        .map_err(EncodeRequestError::Riff)?
        .with_render_timeline(prepared.configuration().render_timeline())
        .map_err(EncodeRequestError::Riff)?;
    let data_bytes =
        u64::try_from(parsed.data_payload_bytes()).map_err(|_| EncodeRequestError::InputExtent)?;
    let prepared = prepared
        .bind_primary_control_rows(data_bytes)
        .map_err(EncodeRequestError::PreparedInput)?;
    validate_external_job_configuration(&prepared)?;

    let configuration = prepared.configuration();
    let codec_rate = configuration.selected_profile().codec_rate();
    let (frame_bytes, total_kbps) = match resolve_encode_request_route(codec_rate, layout)? {
        EncodeRequestRoute::SelectedFivePointOne(_) => (
            codec_rate
                .frame_bytes()
                .ok_or(EncodeRequestError::UnsupportedDataRate)?,
            configuration.bit_rate_kbps(),
        ),
        EncodeRequestRoute::Flat7xI0D0Pair(target) => {
            jocforge_application::source_frame_bridge::build_flat7x_provisional_source_role_map(
                prepared.graph().graph(),
                target,
            )
            .map_err(|error| {
                EncodeRequestError::SourcePipeline(format!("Flat7x input layout: {error:?}"))
            })?;
            (
                usize::try_from(target.aggregate_frame_bytes())
                    .map_err(|_| EncodeRequestError::UnsupportedDataRate)?,
                target
                    .aggregate_rate_kbps()
                    .ok_or(EncodeRequestError::UnsupportedDataRate)?,
            )
        }
    };
    let (frame_bytes, total_kbps) = if let Some(rate) = cmaf_rate {
        validate_cmaf_source(codec_rate, EmdfCarriageProfile::EtsiEmdf)?;
        ((rate * 4) as usize, rate)
    } else if ac3_core {
        validate_ac3_core_rate(codec_rate)?;
        (5120, 1280)
    } else {
        (frame_bytes, total_kbps)
    };
    println!(
        "preflight_only=true\njob_xml_path={}\njob_xml_sha256={}\nconfig_sha256={}\ninput_kind={}\ninput_path={}\ninput_file_sha256={}\ninput_file_bytes={}\nsample_rate_hz={}\nchannels={}\npcm_bits={}\nsamples_per_channel={}\nrender_samples_per_channel={}\ndata_rate_kbps={}\naggregate_rate_kbps={}\nframe_bytes={}\noutput_path={}\noutput_exists=false\noutput_created=false\nmeter_started=false\nsource_callbacks_started=false",
        request.job_xml_path.display(),
        hex_lower(&prepared.job_xml_sha256()),
        hex_lower(&prepared.config_sha256()),
        configuration.input_kind().selected_label(),
        request.input_path.display(),
        hex_lower(&parsed.source_sha256()),
        parsed.resource_bytes(),
        parsed.sample_rate_hz(),
        parsed.channels(),
        parsed.pcm_bits(),
        configuration.render_timeline().file_samples(),
        parsed.total_samples(),
        configuration.bit_rate_kbps(),
        total_kbps,
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
    if matches!(value, "1770-4" | "1770-3" | "1770-2" | "1770-1" | "LeqA") {
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
    let line_drc = configuration.line_drc_profile().xml_value();
    let rf_drc = configuration.rf_drc_profile().xml_value();
    if [line_drc, rf_drc].contains(&"speech") {
        return Err(EncodeRequestError::UnsupportedJobXmlOption(
            "/filter/audio/encode_to_atmos_ddp/drc",
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

    Ok(())
}

fn run_worker(
    input_path: &Path,
    output_path: &Path,
    codec_rate: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    structure: (JocReconstructionLayout, EmdfCarriageProfile),
) -> Result<(), EncodeRequestError> {
    run_worker_with_progress(
        input_path,
        output_path,
        codec_rate,
        authored_job_xml,
        structure,
        None,
        &std::sync::atomic::AtomicBool::new(false),
    )
}

#[derive(Clone, Copy, Debug)]
pub enum EncodePhase {
    Metering,
    Encoding,
}

/// Status events for clients that also display loudness-cache activity.
#[derive(Clone, Copy, Debug)]
pub enum EncodeEvent {
    Progress(EncodePhase, u64, u64),
    CachedLoudness,
    WaitingForLoudness,
}

pub(crate) type EventObserver<'a> = &'a (dyn Fn(EncodeEvent) + Sync);

fn progress_events(progress: ProgressObserver<'_>) -> impl Fn(EncodeEvent) + Sync + '_ {
    move |event| {
        if let EncodeEvent::Progress(phase, done, total) = event {
            progress(phase, done, total);
        }
    }
}

/// Product layouts and aggregate bitrates accepted by the file encoder.
pub const FILE_PRESETS: &[(JocReconstructionLayout, u16, CodecRateProfile)] = &[
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        384,
        CodecRateProfile::Kbps384,
    ),
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        448,
        CodecRateProfile::Kbps448,
    ),
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        576,
        CodecRateProfile::Kbps576,
    ),
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        640,
        CodecRateProfile::Kbps640,
    ),
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        768,
        CodecRateProfile::Kbps768,
    ),
    (
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        1024,
        CodecRateProfile::Kbps1024,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1024,
        CodecRateProfile::Kbps384,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1088,
        CodecRateProfile::Kbps448,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1216,
        CodecRateProfile::Kbps576,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1280,
        CodecRateProfile::Kbps640,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1408,
        CodecRateProfile::Kbps768,
    ),
    (
        JocReconstructionLayout::Flat7x,
        1664,
        CodecRateProfile::Kbps1024,
    ),
];

type ProgressObserver<'a> = &'a (dyn Fn(EncodePhase, u64, u64) + Sync);

/// Encodes an ADM to 5.1 JOC and reports the measured work of each stage.
pub fn encode_five_one_file(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    progress: ProgressObserver<'_>,
) -> Result<(), EncodeRequestError> {
    let layout = JocReconstructionLayout::ExistingSelectedFivePointOne;
    resolve_encode_request_route(rate, layout)?;
    run_worker_with_progress(
        input,
        output,
        Some(rate),
        None,
        (layout, EmdfCarriageProfile::DeployedCompat),
        Some(&progress_events(progress)),
        &std::sync::atomic::AtomicBool::new(false),
    )
}

/// Encodes the experimental 640 + 1024 kbps pair in a private staging directory.
pub fn encode_flat_seven_one_file(
    input: &Path,
    output: &Path,
    progress: ProgressObserver<'_>,
) -> Result<(), EncodeRequestError> {
    let output = normalize_new_output(output)?;
    if output.exists() {
        return Err(EncodeRequestError::OutputAlreadyExists);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (input, progress);
        Err(EncodeRequestError::SourcePipeline(
            "browser encode does not stage a flat7x directory".to_owned(),
        ))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let stage = tempfile::tempdir_in(output.parent().ok_or(EncodeRequestError::InvalidPath)?)
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()))?;
        let candidate = stage.path().join("programme.candidate.ec3");
        run_worker_with_progress(
            input,
            &candidate,
            Some(CodecRateProfile::Kbps1024),
            None,
            (
                JocReconstructionLayout::Flat7x,
                EmdfCarriageProfile::DeployedCompat,
            ),
            Some(&progress_events(progress)),
            &std::sync::atomic::AtomicBool::new(false),
        )?;
        tempfile::TempPath::try_from_path(candidate)
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()))?
            .persist_noclobber(output)
            .map_err(|error| EncodeRequestError::SourcePipeline(error.to_string()))
    }
}

/// Runs the selected preset into a caller-owned staging directory.
pub fn encode_staged_file(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    layout: JocReconstructionLayout,
    progress: ProgressObserver<'_>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    resolve_encode_request_route(rate, layout)?;
    run_worker_with_progress(
        input,
        output,
        Some(rate),
        None,
        (layout, EmdfCarriageProfile::DeployedCompat),
        Some(&progress_events(progress)),
        cancelled,
    )
}

/// Encode a staged file with file-position trimming and silence on the shared XML timeline.
pub fn encode_staged_file_with_timing(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    layout: JocReconstructionLayout,
    timing: &FileRenderTiming,
    progress: ProgressObserver<'_>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    encode_staged_file_with_events(
        input,
        output,
        rate,
        layout,
        timing,
        &progress_events(progress),
        cancelled,
    )
}

/// Encodes a staged file and reports both phase progress and cache activity.
pub fn encode_staged_file_with_events(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    layout: JocReconstructionLayout,
    timing: &FileRenderTiming,
    progress: EventObserver<'_>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    encode_staged_file_with_options(
        input,
        output,
        rate,
        layout,
        (timing, &FileEncodeOptions::default()),
        progress,
        cancelled,
    )
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FileEncodeOptions {
    pub surround_ex: u8,
    pub joc_objects: JocObjectCount,
    pub height_trim_5_1: String,
    pub surround_trim_5_1: String,
    pub metering_mode: String,
    pub dialogue_intelligence: bool,
    pub speech_threshold: u32,
    pub custom_dialnorm: Option<i8>,
    pub line_mode_drc_profile: String,
    pub rf_mode_drc_profile: String,
    pub preferred_downmix_mode: String,
    pub loro_center_mix_level: String,
    pub loro_surround_mix_level: String,
    pub ltrt_center_mix_level: String,
    pub ltrt_surround_mix_level: String,
}

impl Default for FileEncodeOptions {
    fn default() -> Self {
        Self {
            joc_objects: JocObjectCount::Fifteen,
            surround_ex: 1,
            height_trim_5_1: "auto".into(),
            surround_trim_5_1: "auto".into(),
            metering_mode: "1770-4".into(),
            dialogue_intelligence: true,
            speech_threshold: 15,
            custom_dialnorm: None,
            line_mode_drc_profile: "film_light".into(),
            rf_mode_drc_profile: "film_light".into(),
            preferred_downmix_mode: "loro".into(),
            loro_center_mix_level: "-3".into(),
            loro_surround_mix_level: "-3".into(),
            ltrt_center_mix_level: "-3".into(),
            ltrt_surround_mix_level: "-3".into(),
        }
    }
}
/// Encodes a staged file and reports both phase progress and cache activity.
pub fn encode_staged_file_with_options(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    layout: JocReconstructionLayout,
    (timing, options): (&FileRenderTiming, &FileEncodeOptions),
    progress: EventObserver<'_>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    encode_staged_file_with_structure(
        input,
        output,
        rate,
        (layout, EmdfCarriageProfile::DeployedCompat, None),
        (timing, options),
        progress,
        cancelled,
    )
}

/// Encode a staged ADM export with the selected wire structure and normal job events.
pub fn encode_staged_file_with_structure(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    (layout, signalling, topology): (
        JocReconstructionLayout,
        EmdfCarriageProfile,
        Option<SubstreamPlan>,
    ),
    (timing, options): (&FileRenderTiming, &FileEncodeOptions),
    progress: EventObserver<'_>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    if options.joc_objects != JocObjectCount::Fifteen && topology.is_some() {
        return Err(EncodeRequestError::SourcePipeline(
            "Reduced object counts require native stream output".into(),
        ));
    }
    resolve_encode_request_route(rate, layout)?;
    let input = normalize_existing_input(input)?;
    let output = normalize_new_output(output)?;
    let xml = file_job_xml(
        &file_reference_parts(&input)?,
        &file_reference_parts(&output)?,
        rate.kbps().ok_or(EncodeRequestError::UnsupportedDataRate)?,
        timing,
        options,
    );
    run_worker_with_object_count(
        &input,
        &output,
        Some(rate),
        Some(xml.into_bytes()),
        (layout, signalling),
        Some(progress),
        cancelled,
        topology,
        options.joc_objects,
        options.surround_ex,
        None,
    )
    .map(|_| ())
}

fn run_worker_with_progress(
    input_path: &Path,
    output_path: &Path,
    codec_rate: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    structure: (JocReconstructionLayout, EmdfCarriageProfile),
    progress: Option<EventObserver<'_>>,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), EncodeRequestError> {
    run_worker_with_topology(
        input_path,
        output_path,
        codec_rate,
        authored_job_xml,
        structure,
        progress,
        cancelled,
        None,
    )
}

pub fn encode_substream_layout(
    input: &Path,
    output: &Path,
    rate: CodecRateProfile,
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
    topology: SubstreamPlan,
) -> Result<(), EncodeRequestError> {
    resolve_encode_request_route(rate, layout)?;
    run_worker_with_topology(
        input,
        output,
        Some(rate),
        None,
        (layout, signalling),
        None,
        &std::sync::atomic::AtomicBool::new(false),
        Some(topology),
    )
}

pub fn encode_xml_substream_layout(
    path: &Path,
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
    preflight: bool,
    topology: SubstreamPlan,
) -> Result<(), EncodeRequestError> {
    if topology.layout() != layout {
        return Err(EncodeRequestError::UnsupportedJocLayout);
    }
    if preflight {
        return encode_xml_request(
            path,
            layout,
            true,
            signalling,
            topology.ac3_core_lfe().is_some(),
            topology.cmaf_rate(),
        );
    }
    let request = resolve_job_xml_request(path, topology.cmaf_rate().is_some())?;
    run_worker_with_topology(
        &request.input_path,
        &request.output_path,
        None,
        Some(request.job_xml),
        (layout, signalling),
        None,
        &std::sync::atomic::AtomicBool::new(false),
        Some(topology),
    )
}

pub use jocforge_application::spatial_callback::JocObjectCount;

pub fn encode_selected_object_count(
    input: &Path,
    output: &Path,
    object_count: JocObjectCount,
) -> Result<(), EncodeRequestError> {
    encode_object_count_for_layout(
        input,
        output,
        object_count,
        JocReconstructionLayout::ExistingSelectedFivePointOne,
        EmdfCarriageProfile::DeployedCompat,
    )
}

pub fn encode_object_count_for_layout(
    input: &Path,
    output: &Path,
    object_count: JocObjectCount,
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
) -> Result<(), EncodeRequestError> {
    encode_native_object_count(
        input,
        output,
        object_count,
        layout,
        signalling,
        CodecRateProfile::Kbps1024,
    )
}

pub fn encode_native_object_count(
    input: &Path,
    output: &Path,
    object_count: JocObjectCount,
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
    rate: CodecRateProfile,
) -> Result<(), EncodeRequestError> {
    encode_native_with_ex(input, output, object_count, layout, signalling, rate, 1)
}

pub fn encode_native_with_ex(
    input: &Path,
    output: &Path,
    object_count: JocObjectCount,
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
    rate: CodecRateProfile,
    surround_ex: u8,
) -> Result<(), EncodeRequestError> {
    resolve_encode_request_route(rate, layout)?;
    run_worker_with_object_count(
        input,
        output,
        Some(rate),
        None,
        (layout, signalling),
        None,
        &std::sync::atomic::AtomicBool::new(false),
        None,
        object_count,
        surround_ex,
        None,
    )
    .map(|_| ())
}

fn run_worker_with_topology(
    input_path: &Path,
    output_path: &Path,
    codec_rate: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    structure: (JocReconstructionLayout, EmdfCarriageProfile),
    progress: Option<EventObserver<'_>>,
    cancelled: &std::sync::atomic::AtomicBool,
    topology: Option<SubstreamPlan>,
) -> Result<(), EncodeRequestError> {
    run_worker_with_object_count(
        input_path,
        output_path,
        codec_rate,
        authored_job_xml,
        structure,
        progress,
        cancelled,
        topology,
        JocObjectCount::Fifteen,
        1,
        None,
    )
    .map(|_| ())
}

fn run_worker_with_object_count(
    input_path: &Path,
    output_path: &Path,
    codec_rate: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    structure: (JocReconstructionLayout, EmdfCarriageProfile),
    progress: Option<EventObserver<'_>>,
    cancelled: &std::sync::atomic::AtomicBool,
    topology: Option<SubstreamPlan>,
    object_count: JocObjectCount,
    surround_ex: u8,
    resource: Option<std::sync::Arc<[u8]>>,
) -> Result<RunProduct, EncodeRequestError> {
    #[cfg(target_arch = "wasm32")]
    {
        return run_inner(
            input_path,
            output_path,
            codec_rate,
            authored_job_xml,
            structure,
            progress,
            cancelled,
            topology,
            object_count,
            surround_ex,
            resource,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
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
                    structure,
                    progress,
                    cancelled,
                    topology,
                    object_count,
                    surround_ex,
                    resource,
                )
            })
            .map_err(|_| EncodeRequestError::WorkerUnavailable)?;
        worker
            .join()
            .map_err(|_| EncodeRequestError::WorkerPanicked)?
    })
}

fn validate_cmaf_source(
    rate: CodecRateProfile,
    signalling: EmdfCarriageProfile,
) -> Result<(), EncodeRequestError> {
    if rate != CodecRateProfile::Kbps640 || signalling != EmdfCarriageProfile::EtsiEmdf {
        return Err(EncodeRequestError::SourcePipeline(
            "CMAF requires strict EMDF and D0 data_rate 640".into(),
        ));
    }
    Ok(())
}

fn validate_ac3_core_rate(rate: CodecRateProfile) -> Result<(), EncodeRequestError> {
    if rate == CodecRateProfile::Kbps640 {
        Ok(())
    } else {
        Err(EncodeRequestError::SourcePipeline(
            "legacy-ac3 needs a D0 data_rate of 640 kbps (1280 kbps total)".into(),
        ))
    }
}
fn validate_signalling(
    layout: JocReconstructionLayout,
    signalling: EmdfCarriageProfile,
) -> Result<(), EncodeRequestError> {
    if signalling == EmdfCarriageProfile::DeployedCompat
        || (matches!(
            layout,
            JocReconstructionLayout::Flat7x
                | JocReconstructionLayout::FivePointOnePlusHeight
                | JocReconstructionLayout::FivePointOneWithoutPhase
                | JocReconstructionLayout::HeightPairWithPhase
                | JocReconstructionLayout::ExistingSelectedFivePointOne
        ) && signalling == EmdfCarriageProfile::EtsiEmdf)
    {
        Ok(())
    } else {
        Err(EncodeRequestError::UnsupportedJocLayout)
    }
}

fn run_inner(
    input_path: &Path,
    output_path: &Path,
    codec_rate_override: Option<CodecRateProfile>,
    authored_job_xml: Option<Vec<u8>>,
    structure: (JocReconstructionLayout, EmdfCarriageProfile),
    progress: Option<EventObserver<'_>>,
    cancelled: &std::sync::atomic::AtomicBool,
    topology: Option<SubstreamPlan>,
    object_count: JocObjectCount,
    surround_ex: u8,
    resource: Option<std::sync::Arc<[u8]>>,
) -> Result<RunProduct, EncodeRequestError> {
    let (joc_layout, signalling) = structure;
    use jocforge_application::flat7x_pair_source::SurroundExMode;
    let surround_ex = match surround_ex {
        1 => SurroundExMode::NotEncoded,
        0 | 2 if joc_layout != JocReconstructionLayout::Flat7x || topology.is_some() => {
            return Err(EncodeRequestError::SourcePipeline(
                "Surround EX requires native flat7x output".into(),
            ));
        }
        0 => SurroundExMode::NotIndicated,
        2 => SurroundExMode::Encoded,
        _ => {
            return Err(EncodeRequestError::SourcePipeline(
                "Invalid Surround EX mode".into(),
            ));
        }
    };
    validate_signalling(joc_layout, signalling)?;
    if topology
        .as_ref()
        .is_some_and(|plan| plan.layout() != joc_layout)
    {
        return Err(EncodeRequestError::UnsupportedJocLayout);
    }
    eprintln!("Preparing selected ADM input and owner plan...");
    // A caller-supplied resource is the authoritative input; only a path request is normalized.
    let input_path = match resource {
        Some(_) => input_path.to_path_buf(),
        // The browser has no absolute filesystem paths: a resource-less wasm request is served by the
        // host OPFS reader, so the placeholder path is never resolved against a directory.
        #[cfg(target_arch = "wasm32")]
        None => input_path.to_path_buf(),
        #[cfg(not(target_arch = "wasm32"))]
        None => normalize_existing_input(input_path)?,
    };
    #[cfg(not(target_arch = "wasm32"))]
    let output_path = normalize_new_output(output_path)?;
    #[cfg(target_arch = "wasm32")]
    let output_path = output_path.to_path_buf();
    if output_path.exists() {
        return Err(EncodeRequestError::OutputAlreadyExists);
    }
    let cmaf_rate = topology.as_ref().and_then(SubstreamPlan::cmaf_rate);
    if cmaf_rate.is_none() && joc_layout.uses_seven_carriers() {
        validate_flat7x_experimental_candidate_path(&output_path)?;
    }

    let is_external_job_xml = authored_job_xml.is_some();
    let generated_rate = codec_rate_override.unwrap_or(CodecRateProfile::Kbps1024);
    if generated_rate.kbps().is_none() {
        return Err(EncodeRequestError::UnsupportedDataRate);
    }
    let job_xml = if let Some(job_xml) = authored_job_xml {
        job_xml
    } else {
        let input_reference = file_reference_parts(&input_path)?;
        let output_reference = file_reference_parts(&output_path)?;
        let rate_kbps = generated_rate
            .kbps()
            .ok_or(EncodeRequestError::UnsupportedDataRate)?;
        selected_full_file_job_xml(&input_reference, &output_reference, rate_kbps).into_bytes()
    };
    let run_identity = fresh_run_identity()?;
    // A native, non-composed run streams the resource from disk: the walk reads headers and
    // metadata, and the PCM payload is served block by block from the same handle, so the whole ADM
    // is never resident. The composed path still reads it whole: it currently hangs for an
    // unresolved, pre-existing reason documented in docs/memory/agents/implementation-agent.md.
    let resident_resource: std::sync::Arc<[u8]>;
    let streamed_reader: std::sync::Mutex<Box<dyn jocforge_io::pcm::riff_pcm24::RiffDataReader>>;
    let parsed = match resource {
        Some(bytes) => {
            resident_resource = bytes;
            parse_selected_riff_pcm24(&resident_resource, None).map_err(EncodeRequestError::Riff)?
        }
        // The reader is native-only; the browser hands over resident bytes instead.
        #[cfg(not(target_arch = "wasm32"))]
        None if topology.is_none() => {
            streamed_reader = std::sync::Mutex::new(Box::new(
                jocforge_io::pcm::riff_pcm24::FileRiffDataReader::open(&input_path).map_err(
                    |error| EncodeRequestError::InputFile(RiffPcm24FileReadError::from_io(error)),
                )?,
            )
                as Box<dyn jocforge_io::pcm::riff_pcm24::RiffDataReader>);
            parse_selected_riff_pcm24_streamed(&streamed_reader, None)
                .map_err(EncodeRequestError::Riff)?
        }
        // The browser has no filesystem: the host worker serves the resource from OPFS through a
        // synchronous import, so the same streamed walk applies without shared memory.
        #[cfg(target_arch = "wasm32")]
        None => {
            streamed_reader =
                std::sync::Mutex::new(Box::new(jocforge_io::pcm::riff_pcm24::HostRiffDataReader)
                    as Box<dyn jocforge_io::pcm::riff_pcm24::RiffDataReader>);
            parse_selected_riff_pcm24_streamed(&streamed_reader, None)
                .map_err(EncodeRequestError::Riff)?
        }
        #[cfg(not(target_arch = "wasm32"))]
        None => {
            resident_resource =
                read_resource(&input_path).map_err(EncodeRequestError::InputFile)?;
            parse_selected_riff_pcm24(&resident_resource, None).map_err(EncodeRequestError::Riff)?
        }
    };
    let adm = parsed
        .selected_adm_payloads()
        .map_err(EncodeRequestError::Riff)?;
    let wave = SelectedWaveDescriptor::new(
        parsed.sample_rate_hz(),
        parsed.channels(),
        parsed.pcm_bits(),
    );
    let prepared = prepare_selected_job_input(
        run_identity.programme.clone(),
        &job_xml,
        wave,
        parsed.total_samples(),
        ResourceIdentity::new(parsed.source_sha256(), parsed.resource_bytes()),
        SelectedAdmResourcePayloads::new(
            adm.axml(),
            adm.axml_offset(),
            adm.chna(),
            adm.chna_offset(),
            adm.dbmd(),
            adm.dbmd_offset(),
        ),
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
    let parsed = parsed
        .with_adm_channel_projection(
            prepared.graph().graph().bed_mix_sources(),
            prepared.graph().graph().unused_file_channels(),
        )
        .map_err(EncodeRequestError::Riff)?
        .with_render_timeline(prepared.configuration().render_timeline())
        .map_err(EncodeRequestError::Riff)?;
    let data_bytes =
        u64::try_from(parsed.data_payload_bytes()).map_err(|_| EncodeRequestError::InputExtent)?;
    let prepared = prepared
        .bind_primary_control_rows(data_bytes)
        .map_err(EncodeRequestError::PreparedInput)?;
    let selected_profile = prepared.configuration().selected_profile();
    let resolved_joc_profile = resolve_selected_profile(
        &EncodingProfile::selected_with_rate_and_carriage_and_joc_reconstruction_layout(
            selected_profile.codec_rate(),
            selected_profile.carriage(),
            joc_layout,
        ),
    )
    .map_err(|_| EncodeRequestError::UnsupportedJocLayout)?;
    let (frame_bytes, aggregate_rate_kbps) = if resolved_joc_profile
        .joc_reconstruction_layout()
        .uses_seven_carriers()
    {
        let target = Flat7xI0D0TargetPlan::for_layout(selected_profile.codec_rate(), joc_layout)
            .ok_or(EncodeRequestError::UnsupportedJocLayout)?;
        jocforge_application::source_frame_bridge::build_flat7x_provisional_source_role_map(
            prepared.graph().graph(),
            target,
        )
        .map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("Flat7x input layout: {error:?}"))
        })?;
        (
            usize::try_from(target.aggregate_frame_bytes())
                .map_err(|_| EncodeRequestError::UnsupportedDataRate)?,
            target
                .aggregate_rate_kbps()
                .ok_or(EncodeRequestError::UnsupportedDataRate)?,
        )
    } else {
        (
            selected_profile
                .codec_rate()
                .frame_bytes()
                .ok_or(EncodeRequestError::UnsupportedDataRate)?,
            data_rate_kbps,
        )
    };
    let ac3_core = topology
        .as_ref()
        .is_some_and(|plan| plan.ac3_core_lfe().is_some());
    if ac3_core {
        validate_ac3_core_rate(selected_profile.codec_rate())?;
    }
    if cmaf_rate.is_some() {
        validate_cmaf_source(selected_profile.codec_rate(), signalling)?;
    }
    let config_sha256 = prepared.config_sha256();

    eprintln!(
        "prepared input: attempt={} programme={} sha256={} bytes={} sample_rate_hz={} channels={} pcm_bits={} samples_per_channel={} data_rate_kbps={} aggregate_rate_kbps={} frame_bytes={} joc_reconstruction_layout={:?} joc_dmx_config_idx={:?} config_sha256={} output={}",
        run_identity.attempt.value(),
        run_identity.programme.as_str(),
        hex_lower(&parsed.source_sha256()),
        parsed.resource_bytes(),
        parsed.sample_rate_hz(),
        parsed.channels(),
        parsed.pcm_bits(),
        parsed.total_samples(),
        data_rate_kbps,
        cmaf_rate.unwrap_or(if ac3_core { 1280 } else { aggregate_rate_kbps }),
        cmaf_rate.map_or(if ac3_core { 5120 } else { frame_bytes }, |r| (r * 4)
            as usize),
        resolved_joc_profile.joc_reconstruction_layout(),
        resolved_joc_profile.joc_dmx_config_idx(),
        hex_lower(&config_sha256),
        output_path.display(),
    );

    // The existing audio and metadata producers still advance as one transaction.
    // Keep their intermediate elementary stream private when publishing a new topology.
    let native_output = topology.is_none();
    #[cfg(not(target_arch = "wasm32"))]
    // The scratch guard must outlive the encode: dropping it here deletes the directory that the
    // output staging later writes into, which made every composed run fail before its first append.
    let (carrier_path, mut composition, _scratch_guard) = {
        let scratch = topology
            .as_ref()
            .map(|_| {
                tempfile::tempdir_in(
                    output_path
                        .parent()
                        .ok_or(EncodeRequestError::InvalidPath)?,
                )
                .map_err(|e| EncodeRequestError::SourcePipeline(e.to_string()))
            })
            .transpose()?;
        let carrier_path = match &scratch {
            Some(dir) if cmaf_rate.is_some() => dir.path().join("audio.candidate.ec3"),
            Some(dir) => dir.path().join(
                output_path
                    .file_name()
                    .ok_or(EncodeRequestError::InvalidPath)?,
            ),
            None => output_path.clone(),
        };
        let composition = topology
            .map(|plan| {
                tempfile::NamedTempFile::new_in(
                    output_path
                        .parent()
                        .ok_or(EncodeRequestError::InvalidPath)?,
                )
                .map(|file| (plan, file))
                .map_err(|e| EncodeRequestError::SourcePipeline(e.to_string()))
            })
            .transpose()?;
        Ok::<_, EncodeRequestError>((carrier_path, composition, scratch))
    }?;
    #[cfg(target_arch = "wasm32")]
    let (carrier_path, mut composition) = {
        // A composed programme keeps its intermediate carrier out of the namespace: the sink is an
        // owned buffer, so the carrier name only has to stay inside the memory namespace.
        let composition = match topology {
            Some(plan) => Some((plan, ComposedOutput::new(Vec::new()))),
            None => None,
        };
        let carrier_path = if composition.is_some() {
            Path::new("memory://audio.candidate.ec3").to_path_buf()
        } else {
            output_path.clone()
        };
        (carrier_path, composition)
    };
    let final_destination = carrier_path
        .to_str()
        .ok_or(EncodeRequestError::InvalidPath)?;
    let plan = jocforge_application::encode(
        &prepared,
        resolved_joc_profile,
        run_identity.attempt,
        &run_identity.staging_owner,
        final_destination,
    )
    .map_err(|error| {
        #[cfg(target_arch = "wasm32")]
        {
            let detail = jocforge_application::output_publication::admission_rejection_detail(
                &prepared,
                resolved_joc_profile,
                run_identity.attempt,
                &run_identity.staging_owner,
                final_destination,
            )
            .unwrap_or_else(|| "no detail".to_owned());
            return EncodeRequestError::SourcePipeline(format!("{error} | admission={detail}"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        EncodeRequestError::ProductGate(error)
    })?;
    let mut staging = ProgrammeOutputStaging::from_admitted_plan(&plan)
        .map_err(EncodeRequestError::OutputStaging)?;
    let output_binding = plan
        .output_binding()
        .fresh_cold_execution_binding()
        .map_err(|error| {
            EncodeRequestError::OutputStaging(OutputPublicationError::Binding(error))
        })?;

    let assets = unpack_provider_supplements().map_err(EncodeRequestError::MeterPrepass)?;
    let (meter_reference, measurement) = crate::loudness_cache::measure_for_encode(
        &prepared,
        &parsed,
        run_identity.attempt,
        assets.tree(),
        progress,
        cancelled,
    )?;
    let selected_lufs_word = measurement.receipt().selected_lufs_word();
    let selected_lufs = f32::from_bits(selected_lufs_word);
    let drc_root = cold_drc_owner_after_measurement(&prepared, measurement.receipt())
        .map_err(EncodeRequestError::DrcRoot)?;
    let drc_root = if joc_layout.uses_seven_carriers() {
        drc_root
    } else {
        drc_root.with_spatial_receiver_input()
    };
    let source_providers = plan.source_providers();
    let metadata_profile = resolve_selected_profile(
        &EncodingProfile::selected_with_rate_and_carriage_and_joc_reconstruction_layout(
            selected_profile.codec_rate(),
            signalling,
            joc_layout,
        ),
    )
    .map_err(|_| EncodeRequestError::UnsupportedJocLayout)?;
    let metadata_owner = prepare_metadata_owner_from_measurement(
        &prepared,
        &measurement,
        source_providers.metadata_owner().provider_file(),
        source_providers.metadata_owner().wrapper_file(),
        metadata_profile,
    )
    .map_err(EncodeRequestError::MetadataOwner)?;
    eprintln!(
        "programme measurement ready: callbacks={} zero_drains={} selected_lufs_f32={:.6} selected_lufs_bits=0x{selected_lufs_word:08x} selected_lufs_source=ProgrammeMeasurementReceipt.selected_lufs_word selected_lufs_unit=RUST_SELECTED_LUFS_DEE_NATIVE_UNIT_UNBOUND dialnorm={:?} metadata_generation={}",
        measurement.receipt().normal_callbacks(),
        measurement.receipt().zero_drains(),
        selected_lufs,
        drc_root.state().programme_dialnorm(),
        metadata_owner.state_reference().generation(),
    );

    let supplements = assets.tree();
    let spatial_profile = EncodingProfile::selected_with_rate_and_carriage(
        selected_profile.codec_rate(),
        selected_profile.carriage(),
    );
    let main_providers = load_selected_main_spatial_callback_providers_for_profile(
        supplements.child("ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1"),
        supplements.child("F01B-F03W-COLD-STATE-ASSETS-V1"),
        supplements.child("METER-APPROVED-PARAMETER-ASSETS-V1"),
        supplements.child("F04-RECEIVER-APPROVED-PARAMETERS-V1"),
        generate_selected_spatial_math_geometry()
            .map_err(|_| EncodeRequestError::SpatialGeometry)?,
        spatial_profile,
    )
    .map_err(EncodeRequestError::SpatialProviders)?;
    let callback_values = main_providers.callback_values().providers();
    let mut cold_roots = MainSpatialColdRoots::from_selected_providers_for_output(
        &prepared,
        run_identity.attempt,
        &callback_values.f01e,
        &callback_values.f03_target,
        &callback_values.p11_geometry,
        native_output.then_some(metadata_profile),
    )
    .map_err(EncodeRequestError::SpatialColdRoots)?;
    cold_roots.set_joc_object_count(object_count);
    if signalling == EmdfCarriageProfile::EtsiEmdf {
        cold_roots.enable_strict_oamd();
    }
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
    let joc_state = SelectedJocState::cold_with_object_count(object_count);
    let codec_rate = prepared.configuration().selected_profile().codec_rate();
    let codec_root = if resolved_joc_profile
        .joc_reconstruction_layout()
        .uses_seven_carriers()
    {
        CodecOwnerRoot::cold_for_flat7x(codec_rate)
    } else {
        CodecOwnerRoot::cold_for_rate(codec_rate)
    }
    .map_err(|error| EncodeRequestError::SourcePipeline(format!("Codec cold root: {error:?}")))?;
    let model_dialnorm = drc_root.state().programme_dialnorm();
    let measurement_receipt = measurement.receipt();
    let roots = ProgrammeOwnerRoots::ready(
        &prepared,
        resolved_joc_profile,
        run_identity.attempt,
        measurement_receipt,
        &meter_reference,
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

    let flat7x_selected = resolved_joc_profile
        .joc_reconstruction_layout()
        .uses_seven_carriers();
    let mut flat7x_d0_block2_placement_receipts = 0_u64;
    let mut flat7x_d0_block2_max_container_bytes = 0_u16;
    let mut flat7x_d0_block2_overflow_receipts = 0_u64;
    let mut flat7x_emdf_payload_sequence: Option<&'static str> = None;
    let spatial_frames = SpatialFrameProducer {
        prepared: &prepared,
        pcm: &parsed,
        providers: &main_providers,
        attempt: run_identity.attempt,
        normal_count: roots.normal_callbacks(),
        main: Arc::clone(roots.main()),
        reader: Arc::clone(roots.reader_eof()),
        carrier_layout: joc_layout,
    };
    let callback_result = spatial_frames.run(cancelled, |frames, total_callbacks| {
        let mut roots = roots;
        let mut committed_callbacks = 0_u64;
        let mut callback_progress = ProgressBar::new("Programme callbacks");
        callback_progress.update(0, total_callbacks);
        let mut carrier_delay = std::collections::VecDeque::new();
        for callback_value in 0..total_callbacks {
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(EncodeRequestError::Cancelled);
            }
            let callback_value_u32 =
                u32::try_from(callback_value).map_err(|_| EncodeRequestError::ClockMismatch)?;
            let callback_index = FrontendCallbackIndex::new(callback_value_u32);
            let mut pending = frames(callback_value)?;
            if pending.callback_index != callback_index {
                return Err(EncodeRequestError::ClockMismatch);
            }
            let carriers = if flat7x_selected {
                let mix = match pending.flat7x_mix.take() {
                    Some(mix) => mix,
                    None => carrier_delay
                        .back()
                        .map(jocforge_application::source_frame_bridge::Flat7xSceneMix::silence)
                        .ok_or(EncodeRequestError::ClockMismatch)?,
                };
                if carrier_delay.is_empty() {
                    carrier_delay.resize(4, mix.silence());
                }
                carrier_delay.push_back(mix);
                let mut output = carrier_delay
                    .pop_front()
                    .ok_or(EncodeRequestError::ClockMismatch)?;
                if ac3_core {output.align_supplemental_pair_windows(&carrier_delay[0], &carrier_delay[1]);}
                else {output.align_analysis_windows(&carrier_delay[0], &carrier_delay[1]);}
                pending
                    .source_index
                    .map(|source_index| {
                        let extent = Flat7xD0PcmSourceExtent::for_programme_audio(
                            source_index,
                            callback_index,
                            prepared.configuration().source_samples(),
                        )
                        .map_err(|error| {
                            EncodeRequestError::SourcePipeline(format!(
                                "Flat7x audio window: {error:?}"
                            ))
                        })?;
                        Ok::<_, EncodeRequestError>(crate::source_execution::Flat7xCarrierFrame {
                            input_sha256: prepared.resource_identity().input_sha256(),
                            source_extent: extent,
                            mix: output,
                        })
                    })
                    .transpose()?
            } else {
                None
            };
            let callback_transaction_id = pending.transaction_id.clone();

            if let Some(source_index) = pending.source_index {
                let advance = crate::source_execution::stage_source_frame(
                    &prepared,
                    resolved_joc_profile,
                    run_identity.attempt,
                    callback_index,
                    source_index,
                    callback_transaction_id.clone(),
                    pending,
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
                    carriers.as_ref(), composition.as_mut(),frame_bytes,surround_ex,
                )
                .map_err(map_source_frame_stage_error)?;
                let (next_roots, receipt) = roots
                    .commit_source_advance(callback_transaction_id, &prepared, advance)
                    .map_err(EncodeRequestError::SourcePipeline)?;
                roots = next_roots;
                match (
                    receipt.append.as_ref(),
                    receipt.flat7x_d0_block2_placement_receipt.as_ref(),
                ) {
                    (Some(append), Some(placement))
                        if append.durable && append.au_index == placement.au_index().value() =>
                    {
                        let payload_sequence = placement.payload_sequence().label();
                        match flat7x_emdf_payload_sequence {
                            Some(previous) if previous != payload_sequence => {
                                return Err(EncodeRequestError::SourcePipeline(
                                    "Flat7x EMDF payload sequence changed within one programme"
                                        .to_owned(),
                                ));
                            }
                            Some(_) => {}
                            None => {
                                flat7x_emdf_payload_sequence = Some(payload_sequence);
                            }
                        }
                        flat7x_d0_block2_placement_receipts = flat7x_d0_block2_placement_receipts
                            .checked_add(1)
                            .ok_or_else(|| {
                                EncodeRequestError::SourcePipeline(
                                    "D0 block-two placement receipt count overflow".to_owned(),
                                )
                            })?;
                        flat7x_d0_block2_max_container_bytes =
                            flat7x_d0_block2_max_container_bytes.max(placement.container_bytes());
                        if placement.container_bytes() > 511 {
                            flat7x_d0_block2_overflow_receipts = flat7x_d0_block2_overflow_receipts
                                .checked_add(1)
                                .ok_or_else(|| {
                                    EncodeRequestError::SourcePipeline(
                                        "D0 block-two placement overflow count overflow".to_owned(),
                                    )
                                })?;
                        }
                    }
                    (Some(_), None) if flat7x_selected => {
                        return Err(EncodeRequestError::SourcePipeline(
                            "durable Flat7x AU has no D0 block-two placement receipt".to_owned(),
                        ));
                    }
                    (Some(_) | None, None) => {}
                    _ => {
                        return Err(EncodeRequestError::SourcePipeline(
                            "D0 block-two placement receipt differs from its append".to_owned(),
                        ));
                    }
                }
                let flat7x_role_map_assumption = receipt
                    .flat7x_source_role_map
                    .as_ref()
                    .map_or("not_selected", |role_map| {
                        role_map.role_map().assumption_id()
                    });
                committed_callbacks = committed_callbacks.checked_add(1).ok_or_else(|| {
                    EncodeRequestError::SourcePipeline("callback count overflow".to_owned())
                })?;

                if source_index.value() < 2 {
                    eprintln!(
                        "source committed: g={} callback_tx={} source={} source_tx={} disposition=suppressed frame_sha256={} owner_receipts={} phase_generation={}->{}",
                        receipt.callback_index.value(),
                        receipt.callback_transaction_id.as_str(),
                        receipt.source_index.map_or(
                            u64::MAX,
                            jocforge_io::pcm::frontend_pcm_source::CodecSourceIndex::value,
                        ),
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
                        if let Some(d0) = receipt.flat7x_d0_frame_syntax {
                            eprintln!(
                                "Flat7x D0 AU0 syntax receipt: audfrm={}..{} skipflde_bit={} skiple_bit={} skiple=1 skipl_bit={} skipl={} emdf={}..{} nauxbits={}..{} auxdatae_bit={} crcrsv_bit={} crc2={}..{}",
                                d0.audfrm_start_bit,
                                d0.audfrm_end_bit,
                                d0.skipflde_bit_offset,
                                d0.block_two_skiple_bit_offset,
                                d0.block_two_skipl_bit_offset,
                                d0.block_two_container_bytes,
                                d0.block_two_container_start_bit,
                                d0.block_two_container_end_bit,
                                d0.nauxbits_start_bit,
                                d0.nauxbits_end_bit,
                                d0.auxdatae_bit_offset,
                                d0.crcrsv_bit_offset,
                                d0.crc2_start_bit,
                                d0.crc2_end_bit,
                            );
                        }
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
                            {
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    std::fs::metadata(staging.staging_path())
                                        .map_err(|error| {
                                            EncodeRequestError::SourcePipeline(format!(
                                                "staged AU metadata: {error}"
                                            ))
                                        })?
                                        .len()
                                }
                                #[cfg(target_arch = "wasm32")]
                                {
                                    staging.assembled_len()
                                }
                            },
                        );
                    }
                    if append.au_index == 0
                        || append.au_index % 512 == 0
                        || append.au_index + 1 == roots.expected_published()
                    {
                        eprintln!(
                            "AU append committed: g={} callback_tx={} source={} source_tx={} au={} frame_bytes={} crc_residual={} durable={} published={}/{} role_map_assumption={} phase_generation={}->{}",
                            receipt.callback_index.value(),
                            receipt.callback_transaction_id.as_str(),
                            receipt.source_index.map_or(
                                u64::MAX,
                                jocforge_io::pcm::frontend_pcm_source::CodecSourceIndex::value,
                            ),
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
                            flat7x_role_map_assumption,
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
                let (next_roots, receipt) = roots
                    .commit_callback_only(
                        &prepared,
                        callback_index,
                        callback_transaction_id,
                        pending.main_roots,
                        pending.reader_eof_root,
                        pending.receipts,
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
            if let Some(report) = progress {
                report(EncodeEvent::Progress(EncodePhase::Encoding, callback_value + 1, total_callbacks));
            } else {
                callback_progress.update(callback_value + 1, total_callbacks);
            }
        }
        callback_progress.finish();

        if committed_callbacks != total_callbacks || !roots.is_complete_for_eof() {
            return Err(EncodeRequestError::SourcePipeline(
                "callback/source/AU clocks did not reach the selected EOF boundary".to_owned(),
            ));
        }
        // Completing the callbacks is not publication: the last progress observer can cancel.
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(EncodeRequestError::Cancelled);
        }
        Ok((roots, committed_callbacks))
    });
    let (mut roots, committed_callbacks) = match callback_result {
        Ok(value) => value,
        Err(error) => {
            staging.abandon_attempt();
            return Err(error);
        }
    };
    if flat7x_selected {
        if flat7x_d0_block2_placement_receipts != roots.published_aus() {
            staging.abandon_attempt();
            return Err(EncodeRequestError::SourcePipeline(format!(
                "D0 block-two placement receipts {flat7x_d0_block2_placement_receipts}/{} do not match published AUs",
                roots.published_aus(),
            )));
        }
        if flat7x_emdf_payload_sequence != Some("flat7x_experimental_joc_pair_v1") {
            staging.abandon_attempt();
            return Err(EncodeRequestError::SourcePipeline(
                "Flat7x candidate did not retain the diagnostic JOC-pair payload sequence"
                    .to_owned(),
            ));
        }
    }
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
    let (finalizing_roots, _, finalize_proposal) =
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
    #[cfg(not(target_arch = "wasm32"))]
    if let Some((_, mut file)) = composition {
        roots
            .output()
            .validate_finalize_proposal(&finalize_proposal)
            .map_err(|e| {
                EncodeRequestError::SourcePipeline(format!("source finalization: {e:?}"))
            })?;
        if file
            .as_file()
            .metadata()
            .map_err(|e| EncodeRequestError::SourcePipeline(e.to_string()))?
            .len()
            != if let Some(rate) = cmaf_rate {
                roots.published_aus() * u64::from(rate) * 4
            } else if ac3_core {
                roots.published_aus() * 5120
            } else {
                roots.durable_bytes()
            }
        {
            return Err(EncodeRequestError::SourcePipeline(
                "composed output length mismatch".into(),
            ));
        }
        let file = if let Some(rate) = cmaf_rate {
            use std::io::{BufReader, BufWriter, Seek};
            let io_error = |e: std::io::Error| {
                EncodeRequestError::SourcePipeline(format!("CMAF packaging: {e}"))
            };
            file.as_file_mut().rewind().map_err(io_error)?;
            let mut packaged = tempfile::NamedTempFile::new_in(
                output_path
                    .parent()
                    .ok_or(EncodeRequestError::InvalidPath)?,
            )
            .map_err(io_error)?;
            let count = u32::try_from(roots.published_aus())
                .map_err(|_| EncodeRequestError::ClockMismatch)?;
            crate::cmaf::package(
                BufReader::new(file.as_file_mut()),
                BufWriter::new(packaged.as_file_mut()),
                joc_layout,
                rate,
                count,
            )
            .map_err(io_error)?;
            packaged
        } else {
            file
        };
        file.as_file()
            .sync_all()
            .map_err(|e| EncodeRequestError::SourcePipeline(e.to_string()))?;
        staging.abandon_attempt();
        file.persist_noclobber(&output_path)
            .map_err(|e| EncodeRequestError::SourcePipeline(e.to_string()))?;
        eprintln!(
            "composed {} access units: {}",
            roots.published_aus(),
            output_path.display()
        );
        return Ok(RunProduct::Published);
    }
    if resolved_joc_profile
        .joc_reconstruction_layout()
        .uses_seven_carriers()
    {
        let payload_sequence = flat7x_emdf_payload_sequence.ok_or_else(|| {
            EncodeRequestError::SourcePipeline(
                "Flat7x experimental candidate has no payload-sequence receipt".to_owned(),
            )
        })?;
        let candidate = match capture_flat7x_experimental_candidate(
            &mut staging,
            roots.output(),
            finalize_proposal,
        ) {
            Ok(candidate) => candidate,
            Err(error) => {
                staging.abandon_attempt();
                return Err(EncodeRequestError::SourcePipeline(format!(
                    "Flat7x experimental candidate capture failed: {error}"
                )));
            }
        };
        eprintln!(
            "experimental candidate captured: status=EXPERIMENTAL_CANDIDATE output={} bytes={} sha256={} attempt={} emdf_payload_sequence={} d0_block_two_placement_receipts={}/{} d0_block_two_container_max_C={} overflow_receipts={} production_allowed={} final_product_exposure=false programme_done=false render=NOT_RUN playback=NOT_RUN",
            candidate.candidate_path().display(),
            candidate.bytes_written(),
            hex_lower(candidate.sha256()),
            candidate.attempt_id(),
            payload_sequence,
            flat7x_d0_block2_placement_receipts,
            roots.published_aus(),
            flat7x_d0_block2_max_container_bytes,
            flat7x_d0_block2_overflow_receipts,
            candidate.production_allowed(),
        );
        // The captured native carrier is the product only when no composition was requested.
        #[cfg(target_arch = "wasm32")]
        {
            if let Some((_, sink)) = composition {
                let bytes = sink.into_inner();
                if bytes.len() as u64 != roots.durable_bytes() {
                    return Err(EncodeRequestError::SourcePipeline(
                        "composed output length mismatch".into(),
                    ));
                }
                return Ok(RunProduct::Bytes(bytes));
            }
            return staging
                .staged_stream_bytes()
                .map(RunProduct::Bytes)
                .ok_or_else(|| {
                    EncodeRequestError::SourcePipeline(
                        "Flat7x memory candidate has no complete stream".to_owned(),
                    )
                });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = &candidate;
            return Ok(RunProduct::Published);
        }
    }

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
    // The file targets cross-check the terminal receipt against the exposed path. A target
    // without a namespace has no second copy to stat: its receipt was minted from the assembled
    // buffer, which is the product.
    #[cfg(not(target_arch = "wasm32"))]
    {
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
    // The file targets expose the product through their namespace; a target without one hands
    // the assembled buffer back to the caller instead of inventing a path.
    #[cfg(target_arch = "wasm32")]
    // A composed programme is published through the composition sink, not the carrier staging.
    #[cfg(target_arch = "wasm32")]
    let product = match composition {
        Some((_, sink)) => RunProduct::Bytes(sink.into_inner()),
        None => RunProduct::Bytes(staging.into_product_bytes()),
    };
    #[cfg(not(target_arch = "wasm32"))]
    let product = RunProduct::Published;
    #[cfg(not(target_arch = "wasm32"))]
    let product = RunProduct::Published;
    Ok(product)
}

struct SpatialFrameProducer<'a> {
    prepared: &'a PreparedJobInput,
    pcm: &'a jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'a>,
    providers: &'a crate::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
    attempt: AttemptId,
    normal_count: jocforge_io::pcm::frontend_pcm_source::NormalCallbackCount,
    main: Arc<MainSpatialColdRoots>,
    reader: Arc<ReaderEofMachine>,
    carrier_layout: JocReconstructionLayout,
}

type SpatialFrameResult = Result<SpatialFrameAdvance, EncodeRequestError>;

#[cfg(not(target_arch = "wasm32"))]
fn send_spatial_frames(
    ready: std::sync::mpsc::SyncSender<SpatialFrameResult>,
    total: u64,
    cancelled: &std::sync::atomic::AtomicBool,
    mut next: impl FnMut(u64) -> SpatialFrameResult,
) {
    for index in 0..total {
        // A consumer can already be waiting in recv when cancellation becomes visible here.
        let result = if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            Err(EncodeRequestError::Cancelled)
        } else {
            next(index)
        };
        let failed = result.is_err();
        if ready.send(result).is_err() || failed {
            break;
        }
    }
}

impl SpatialFrameProducer<'_> {
    fn run<T>(
        mut self,
        cancelled: &std::sync::atomic::AtomicBool,
        consume: impl FnOnce(
            &mut dyn FnMut(u64) -> SpatialFrameResult,
            u64,
        ) -> Result<T, EncodeRequestError>,
    ) -> Result<T, EncodeRequestError> {
        let total = self
            .normal_count
            .value()
            .checked_add(4)
            .ok_or(EncodeRequestError::ClockMismatch)?;
        // The browser isolate has no producer thread, so the consumer pulls each frame from this
        // producer in order. Nothing is buffered ahead, so memory tracks one frame rather than the
        // whole programme.
        #[cfg(target_arch = "wasm32")]
        {
            let _ = cancelled;
            let mut take = |index: u64| self.next(index);
            return consume(&mut take, total);
        }
        // A single lookahead slot keeps DSP memory bounded; only the consumer publishes and commits.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (ready, frames) = std::sync::mpsc::sync_channel(1);
            thread::scope(|scope| {
                let worker = thread::Builder::new()
                    .name("jocforge-spatial-frames".into())
                    .stack_size(ENCODE_REQUEST_STACK_BYTES)
                    .spawn_scoped(scope, move || {
                        send_spatial_frames(ready, total, cancelled, |index| self.next(index));
                    })
                    .map_err(|_| EncodeRequestError::WorkerUnavailable)?;
                let mut take = |_index: u64| {
                    frames
                        .recv()
                        .map_err(|_| EncodeRequestError::WorkerPanicked)
                        .and_then(|result| result)
                };
                let result = consume(&mut take, total);
                // Release the receiver before joining: a producer blocked in `send` only wakes when
                // the channel disconnects, and joining first deadlocked every failed composed run.
                drop(take);
                drop(frames);
                worker
                    .join()
                    .map_err(|_| EncodeRequestError::WorkerPanicked)?;
                result
            })
        }
    }

    fn next(&mut self, index: u64) -> SpatialFrameResult {
        let callback = FrontendCallbackIndex::new(
            u32::try_from(index).map_err(|_| EncodeRequestError::ClockMismatch)?,
        );
        let pending = if index < self.normal_count.value() {
            stage_main_callback_candidate(
                self.prepared,
                self.pcm,
                self.attempt,
                callback,
                &self.main,
                &self.reader,
                self.providers,
                self.carrier_layout,
            )?
        } else {
            stage_main_cached_callback_candidate(
                self.prepared,
                self.attempt,
                callback,
                &self.main,
                &self.reader,
                self.normal_count,
                self.providers,
            )?
        };
        let frame = pending.finish(self.prepared, &self.main, &self.reader)?;
        self.main = Arc::clone(&frame.main_roots);
        self.reader = Arc::clone(&frame.reader_eof_root);
        Ok(frame)
    }
}

pub(crate) struct SpatialFrameAdvance {
    pub callback_index: FrontendCallbackIndex,
    pub source_index: Option<CodecSourceIndex>,
    flat7x_mix: Option<jocforge_application::source_frame_bridge::Flat7xSceneMix>,
    pub spatial: MainSpatialFrame,
    pub main_roots: Arc<MainSpatialColdRoots>,
    pub reader_eof_root: Arc<ReaderEofMachine>,
    pub receipts: [jocforge_application::independent_meter_callback::GtcOwnerReceipt; 4],
    transaction_id: CallbackTransactionId,
}

struct PendingMainCallback {
    callback_index: FrontendCallbackIndex,
    source_index: Option<CodecSourceIndex>,
    flat7x_mix: Option<jocforge_application::source_frame_bridge::Flat7xSceneMix>,
    frontend: Option<FrontendControlReplacementCandidate>,
    spatial: MainSpatialCallbackCandidate,
    reader_eof_candidate: ReaderEofReplacementCandidate,
    transaction_id: CallbackTransactionId,
}

impl PendingMainCallback {
    fn finish(
        self,
        prepared: &PreparedJobInput,
        before: &MainSpatialColdRoots,
        reader_before: &ReaderEofMachine,
    ) -> Result<SpatialFrameAdvance, EncodeRequestError> {
        let parent = if let Some(frontend) = self.frontend {
            before.prepare_spatial_after_callback(
                prepared,
                frontend,
                self.spatial,
                self.transaction_id.clone(),
            )
        } else {
            before.prepare_spatial_after_cached_callback(
                prepared,
                self.spatial,
                self.transaction_id.clone(),
            )
        }
        .map_err(|e| {
            EncodeRequestError::SourcePipeline(format!(
                "spatial callback {}: {e:?}",
                self.callback_index.value()
            ))
        })?;
        if parent.transaction_id() != &self.transaction_id
            || parent.callback_index() != self.callback_index
            || self.reader_eof_candidate.owner_receipt().transaction_id()
                != self.transaction_id.as_str()
            || self
                .reader_eof_candidate
                .owner_receipt()
                .before_generation()
                != reader_before.generation()
            || self.reader_eof_candidate.gtc_receipt().owner_id()
                != jocforge_application::reader_eof_callback::GtcOwnerId::ReaderEof
        {
            return Err(EncodeRequestError::ClockMismatch);
        }
        let (main, spatial, [frontend, control, audio]) = parent.into_frame_parts();
        let (reader, _, reader_receipt) = self.reader_eof_candidate.into_parts();
        Ok(SpatialFrameAdvance {
            callback_index: self.callback_index,
            source_index: self.source_index,
            transaction_id: self.transaction_id,
            flat7x_mix: self.flat7x_mix,
            spatial,
            main_roots: Arc::new(main),
            reader_eof_root: Arc::new(reader),
            receipts: [frontend, control, audio, reader_receipt],
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "The callback stage keeps its owner roots, providers, and callback controls explicit"
)]
fn stage_main_callback_candidate(
    prepared: &PreparedJobInput,
    parsed: &jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24<'_>,
    attempt: AttemptId,
    callback_index: FrontendCallbackIndex,
    roots: &MainSpatialColdRoots,
    reader_eof_before: &ReaderEofMachine,
    providers: &crate::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
    carrier_layout: JocReconstructionLayout,
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
    let source_index = frame.source_index();
    let flat7x_mix = if carrier_layout.uses_seven_carriers() {
        let previous = roots.frontend_control().committed().dynamic_after_state();
        let mix = jocforge_application::source_frame_bridge::Flat7xSceneMix::render_for_layout(
            carrier_layout,
            prepared.graph().graph(),
            frame.physical_pcm_rows(),
            controls.physical_rows(),
            previous.map(|state| state.physical_rows()),
        )
        .map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("Flat7x carrier mix: {error:?}"))
        })?;
        Some(mix)
    } else {
        None
    };
    let values = providers.callback_values().providers();
    let spatial = stage_main_spatial_callback_on_current_thread(MainSpatialCallbackInput {
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
        source_index,
        flat7x_mix,
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
    roots: &MainSpatialColdRoots,
    reader_eof: &ReaderEofMachine,
    normal_callbacks: jocforge_io::pcm::frontend_pcm_source::NormalCallbackCount,
    providers: &crate::selected_spatial_callback_providers::SelectedMainSpatialCallbackProviders,
) -> Result<PendingMainCallback, EncodeRequestError> {
    let source_index = source_for_callback_clock(callback_index);
    let transaction_id = CallbackTransactionId::new(&format!(
        "jocforge-main-callback-g{}",
        callback_index.value()
    ))
    .map_err(|_| EncodeRequestError::CallbackTransaction)?;
    let callback_lineage =
        CallbackLineageIdentity::new(prepared.programme_id().clone(), attempt, callback_index);
    let frontend_control = roots.frontend_control();
    let after = frontend_control
        .committed()
        .dynamic_after_state()
        .ok_or_else(|| {
            EncodeRequestError::SourcePipeline("MAIN F01E after-state unavailable".to_owned())
        })?;
    let held_controls = after
        .held_source_view_for_cached_callback(callback_lineage)
        .map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("MAIN cached F01E held controls: {error:?}"))
        })?;
    let reader_eof_candidate = stage_reader_eof_cached_drain(
        reader_eof,
        callback_index,
        normal_callbacks,
        transaction_id.clone(),
    )
    .map_err(|error| {
        EncodeRequestError::SourcePipeline(format!("ReaderEof cached drain: {error:?}"))
    })?;

    let values = providers.callback_values().providers();
    let spatial =
        stage_main_spatial_cached_callback_on_current_thread(MainSpatialCachedCallbackInput {
            prepared,
            controls: MainHeldControlOutput::AdmInput(held_controls),
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
        source_index,
        flat7x_mix: None,
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
    std::fs::canonicalize(path).map_err(|error| {
        EncodeRequestError::SourcePipeline(format!(
            "could not resolve input file {}: {error}",
            path.display()
        ))
    })
}

fn normalize_new_output(path: &Path) -> Result<PathBuf, EncodeRequestError> {
    if !path.is_absolute() {
        return Err(EncodeRequestError::InvalidPath);
    }
    let file_name = path.file_name().ok_or(EncodeRequestError::InvalidPath)?;
    let parent = path.parent().ok_or(EncodeRequestError::InvalidPath)?;
    let normalized_parent = std::fs::canonicalize(parent).map_err(|error| {
        EncodeRequestError::SourcePipeline(format!(
            "could not resolve output directory {}: {error}",
            parent.display()
        ))
    })?;
    Ok(normalized_parent.join(file_name))
}

fn validate_flat7x_experimental_candidate_path(path: &Path) -> Result<(), EncodeRequestError> {
    // A browser target publishes into memory, so there is no candidate file name or attempt
    // directory to validate. The layout and extent checks still run in the capture path.
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        return Ok(());
    }
    let candidate_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(EncodeRequestError::InvalidPath)?;
    if !candidate_name.ends_with(".candidate.ec3") {
        return Err(EncodeRequestError::ExperimentalCandidatePathRequired);
    }
    let parent = path
        .parent()
        .filter(|parent| parent.is_absolute())
        .ok_or(EncodeRequestError::InvalidPath)?;
    let mut entries = std::fs::read_dir(parent).map_err(|_| EncodeRequestError::InvalidPath)?;
    if entries
        .next()
        .transpose()
        .map_err(|_| EncodeRequestError::InvalidPath)?
        .is_some()
    {
        return Err(EncodeRequestError::ExperimentalCandidateDirectoryNotEmpty);
    }
    Ok(())
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
    file_job_xml(
        input,
        output,
        data_rate_kbps,
        &FileRenderTiming::default(),
        &FileEncodeOptions::default(),
    )
}

fn file_job_xml(
    input: &FileReferenceParts,
    output: &FileReferenceParts,
    data_rate_kbps: u32,
    timing: &FileRenderTiming,
    options: &FileEncodeOptions,
) -> String {
    let field = |s: &str, default: &str| {
        if s.trim().is_empty() {
            default.to_string()
        } else {
            xml_escape(s.trim())
        }
    };
    let rate = field(&timing.frame_rate, "not_indicated");
    let start = field(&timing.start, "");
    let end = field(&timing.end, "");
    let prepend = field(&timing.prepend_silence, "0.0");
    let append = field(&timing.append_silence, "0.0");
    let dialogue_intelligence = options.dialogue_intelligence;
    let speech_threshold = options.speech_threshold;
    let custom_dialnorm = options.custom_dialnorm.map_or_else(String::new, |level| {
        format!("<custom_dialnorm>{level}</custom_dialnorm>")
    });
    let height_trim_5_1 = xml_escape(&options.height_trim_5_1);
    let surround_trim_5_1 = xml_escape(&options.surround_trim_5_1);
    let metering_mode = xml_escape(&options.metering_mode);
    let line_mode_drc_profile = xml_escape(&options.line_mode_drc_profile);
    let rf_mode_drc_profile = xml_escape(&options.rf_mode_drc_profile);
    let preferred_downmix_mode = xml_escape(&options.preferred_downmix_mode);
    let loro_center_mix_level = xml_escape(&options.loro_center_mix_level);
    let loro_surround_mix_level = xml_escape(&options.loro_surround_mix_level);
    let ltrt_center_mix_level = xml_escape(&options.ltrt_center_mix_level);
    let ltrt_surround_mix_level = xml_escape(&options.ltrt_surround_mix_level);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><job_config><input><audio><adm><file_name>{}</file_name><storage><local><path>{}</path></local></storage><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa></adm></audio></input><filter><audio><encode_to_atmos_ddp><loudness><measure_only><metering_mode>{metering_mode}</metering_mode><dialogue_intelligence>{dialogue_intelligence}</dialogue_intelligence><speech_threshold>{speech_threshold}</speech_threshold></measure_only></loudness><data_rate>{}</data_rate><timecode_frame_rate>{rate}</timecode_frame_rate><start>{start}</start><end>{end}</end><time_base>file_position</time_base><prepend_silence_duration>{prepend}</prepend_silence_duration><append_silence_duration>{append}</append_silence_duration><drc><line_mode_drc_profile>{line_mode_drc_profile}</line_mode_drc_profile><rf_mode_drc_profile>{rf_mode_drc_profile}</rf_mode_drc_profile></drc><downmix><loro_center_mix_level>{loro_center_mix_level}</loro_center_mix_level><loro_surround_mix_level>{loro_surround_mix_level}</loro_surround_mix_level><ltrt_center_mix_level>{ltrt_center_mix_level}</ltrt_center_mix_level><ltrt_surround_mix_level>{ltrt_surround_mix_level}</ltrt_surround_mix_level><preferred_downmix_mode>{preferred_downmix_mode}</preferred_downmix_mode></downmix><custom_trims><surround_trim_5_1>{surround_trim_5_1}</surround_trim_5_1><height_trim_5_1>{height_trim_5_1}</height_trim_5_1></custom_trims>{custom_dialnorm}</encode_to_atmos_ddp></audio></filter><output><ec3><file_name>{}</file_name><storage><local><path>{}</path></local></storage></ec3></output></job_config>",
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

/// Distinguishes concurrent requests inside one process or isolate.
///
/// The native targets read the wall clock. A browser isolate has no clock, so it counts attempts
/// instead; the nonce never reaches the bitstream, it only separates staging owners.
#[cfg(not(target_arch = "wasm32"))]
fn run_nonce() -> Result<u128, EncodeRequestError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .map_err(|_| EncodeRequestError::ProgrammeIdentity)
}

#[cfg(target_arch = "wasm32")]
fn run_nonce() -> Result<u128, EncodeRequestError> {
    static NEXT_ATTEMPT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    Ok(u128::from(
        NEXT_ATTEMPT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    ))
}

/// The process identity used only to keep attempt labels distinct.
///
/// A browser isolate has no process table, so it reports a fixed owner number; uniqueness there
/// comes from the attempt counter above.
#[cfg(not(target_arch = "wasm32"))]
fn run_process_id() -> u32 {
    std::process::id()
}

#[cfg(target_arch = "wasm32")]
fn run_process_id() -> u32 {
    0
}

fn fresh_run_identity() -> Result<CliRunIdentity, EncodeRequestError> {
    let nonce = run_nonce()?;
    let attempt = u64::try_from(nonce).map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let unique_label = format!("jocforge-cli-{}-{nonce}", run_process_id());
    let programme_id =
        ProgrammeId::new(&unique_label).map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    let staging_owner_id = OutputStagingOwnerId::new(&format!(
        "jocforge-output-owner-{}-{nonce}",
        run_process_id()
    ))
    .map_err(|_| EncodeRequestError::ProgrammeIdentity)?;
    Ok(CliRunIdentity {
        programme: programme_id,
        attempt: AttemptId::new(attempt),
        staging_owner: staging_owner_id,
    })
}

pub(crate) fn hex_lower(bytes: &[u8]) -> String {
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

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cancelled_spatial_producer_sends_a_terminal_result() {
        use std::sync::{atomic::AtomicBool, mpsc::sync_channel};

        let (ready, frames) = sync_channel(1);
        let worker = std::thread::spawn(move || {
            super::send_spatial_frames(ready, 2, &AtomicBool::new(true), |_| {
                panic!("a cancelled producer must not start another spatial frame")
            });
        });
        assert!(matches!(
            frames.recv(),
            Ok(Err(EncodeRequestError::Cancelled))
        ));
        assert!(frames.recv().is_err());
        worker.join().unwrap();
    }

    #[test]
    fn generated_job_uses_dialogue_defaults_and_honours_saved_options() {
        use super::{FileEncodeOptions, FileRenderTiming, file_job_xml};
        let path = FileReferenceParts {
            file_name: "test.wav".into(),
            local_path: ".".into(),
        };
        let default_xml = selected_full_file_job_xml(&path, &path, 768);
        assert!(default_xml.contains("<dialogue_intelligence>true</dialogue_intelligence><speech_threshold>15</speech_threshold>"));
        let saved: FileEncodeOptions =
            serde_json::from_str(r#"{"dialogue_intelligence":false,"speech_threshold":72}"#)
                .unwrap();
        let xml = file_job_xml(&path, &path, 768, &FileRenderTiming::default(), &saved);
        assert!(xml.contains("<dialogue_intelligence>false</dialogue_intelligence><speech_threshold>72</speech_threshold>"));
    }

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
    fn default_cli_and_public_file_options_deliver_the_authored_trim_pair() {
        use super::{FileEncodeOptions, FileRenderTiming, SelectedWaveDescriptor, file_job_xml};
        use jocforge_domain::frontend::job_configuration::parse_selected_job_configuration;

        let input = FileReferenceParts {
            file_name: "in.wav".into(),
            local_path: ".".into(),
        };
        let output = FileReferenceParts {
            file_name: "out.ec3".into(),
            local_path: ".".into(),
        };
        let parse = |xml: String| {
            parse_selected_job_configuration(
                xml.as_bytes(),
                SelectedWaveDescriptor::new(48000, 12, 24),
                4608,
            )
            .unwrap()
        };
        let cli = parse(selected_full_file_job_xml(&input, &output, 1024));
        assert_eq!(cli.surround_trim_5_1().xml_value(), "auto");
        assert_eq!(cli.height_trim_5_1().xml_value(), "auto");
        let route = resolve_encode_request_route(
            CodecRateProfile::Kbps1024,
            JocReconstructionLayout::ExistingSelectedFivePointOne,
        )
        .unwrap();
        assert_eq!(
            route,
            EncodeRequestRoute::SelectedFivePointOne(cli.selected_profile())
        );
        for (s, h) in [
            ("auto", "auto"),
            ("0", "auto"),
            ("auto", "-3"),
            ("-6", "-9"),
        ] {
            let options: FileEncodeOptions = serde_json::from_value(serde_json::json!({
                "surround_trim_5_1": s, "height_trim_5_1": h
            }))
            .unwrap();
            let job = parse(file_job_xml(
                &input,
                &output,
                1024,
                &FileRenderTiming::default(),
                &options,
            ));
            assert_eq!(job.surround_trim_5_1().xml_value(), s);
            assert_eq!(job.height_trim_5_1().xml_value(), h);
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
    fn flat7x_presets_bind_aggregate_and_d0_rate() {
        for &(layout, total, rate) in super::FILE_PRESETS {
            if !layout.uses_seven_carriers() {
                continue;
            }
            let EncodeRequestRoute::Flat7xI0D0Pair(target) =
                resolve_encode_request_route(rate, layout).unwrap()
            else {
                panic!("expected pair");
            };
            assert_eq!(target.aggregate_rate_kbps(), Some(u32::from(total)));
            assert_eq!(target.d0().rate(), rate);
        }
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
    fn flat7x_route_enters_input_admission_and_uses_the_d0_rate_row() {
        let EncodeRequestRoute::Flat7xI0D0Pair(target) = resolve_encode_request_route(
            CodecRateProfile::Kbps1024,
            JocReconstructionLayout::Flat7x,
        )
        .unwrap() else {
            panic!("the Flat7x selector resolves the approved I0/D0 target");
        };
        assert_eq!(target.i0().rate(), CodecRateProfile::Kbps640);
        assert_eq!(target.d0().rate(), CodecRateProfile::Kbps1024);
        assert_eq!(target.aggregate_rate_kbps(), Some(1_664));

        assert!(matches!(
            run_with_data_rate_and_joc_layout(
                Path::new("missing-input.wav"),
                Path::new("unused-output.ec3"),
                CodecRateProfile::Kbps1024,
                JocReconstructionLayout::Flat7x,
            ),
            Err(EncodeRequestError::InvalidPath)
        ));
    }

    #[test]
    fn selected_cli_gate_rejects_nondefault_metering_modes() {
        assert!(ensure_selected_metering_mode("1770-4").is_ok());
        for mode in ["1770-3", "1770-2", "1770-1", "LeqA"] {
            assert!(ensure_selected_metering_mode(mode).is_ok());
        }
        for mode in ["", "unknown"] {
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
